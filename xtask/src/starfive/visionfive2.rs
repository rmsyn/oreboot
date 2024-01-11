use crate::util::{
    compile_board_dt, dist_dir, find_binutils_prefix_or_fail, get_cargo_cmd_in, objcopy,
    project_root,
};
use crate::{layout_flash, Cli, Commands, Env};
// use fdt;
use log::{error, info, trace};
use std::{
    fs::{self, File},
    io::{self, Seek, SeekFrom},
    path::Path,
    process,
};

extern crate layoutflash;
use layoutflash::areas::{create_areas, Area};

use super::visionfive2_hdr::spl_create_hdr;

const SRAM0_SIZE: u64 = 64 * 1024;

const ARCH: &str = "riscv64";
const TARGET: &str = "riscv64imac-unknown-none-elf";

const BT0_BIN: &str = "starfive-visionfive2-bt0.bin";
const BT0_ELF: &str = "starfive-visionfive2-bt0";

const MAIN_BIN: &str = "starfive-visionfive2-main.bin";
const MAIN_ELF: &str = "starfive-visionfive2-main";

const BOARD_DTB: &str = "starfive-visionfive2-board.dtb";

// TODO: find great names
const FDT_BIN: &str = "starfive-visionfive2-board.fdtbin";
const SRAM_IMAGE: &str = "starfive-visionfive2-sram.bin";

const IMAGE_BIN: &str = "starfive-visionfive2.bin";

pub(crate) fn execute_command(args: &Cli, features: Vec<String>) {
    match args.command {
        Commands::Make => {
            info!("building VisionFive2");
            // Get binutils first so we can fail early
            let binutils_prefix = &find_binutils_prefix_or_fail(ARCH);
            // Build the stages - should we parallelize this?
            xtask_build_jh7110_main(&args.env);
            xtask_build_jh7110_bt0(&args.env, &features);

            objcopy(&args.env, binutils_prefix, TARGET, ARCH, BT0_ELF, BT0_BIN);
            objcopy(&args.env, binutils_prefix, TARGET, ARCH, MAIN_ELF, MAIN_BIN);
            // dtb
            compile_board_dt(&args.env, TARGET, &board_project_root(), BOARD_DTB);
            xtask_build_dtb_image(&args.env);
            // stitch it all together
            xtask_concat_binaries(&args.env);
            // add funny header :-)
            xtask_add_bt0_header(&args.env);
        }
        _ => {
            error!("command {:?} not implemented", args.command);
        }
    }
}

fn xtask_add_bt0_header(env: &Env) {
    let cwd = dist_dir(env, TARGET);
    let bin_path = cwd.join(IMAGE_BIN);
    trace!("add wacky header to {bin_path:?}");
    let dat = fs::read(bin_path).expect("SRAM image");
    let out = spl_create_hdr(dat);
    let out_path = cwd.join(SRAM_IMAGE);
    fs::write(out_path.clone(), out).expect("writing output");

    println!("======= DONE =======");
    println!("Output file: {:?}", &out_path.into_os_string());
}

fn xtask_build_jh7110_bt0(env: &Env, features: &Vec<String>) {
    trace!("build JH7110 bt0");
    let mut command = get_cargo_cmd_in(env, board_project_root(), "bt0", "build");
    if features.len() != 0 {
        let command_line_features = features.join(",");
        trace!("append command line features: {command_line_features}");
        command.arg("--no-default-features");
        command.args(&["--features", &command_line_features]);
    } else {
        trace!("no command line features appended");
    }
    let status = command.status().unwrap();
    trace!("cargo returned {status}");
    if !status.success() {
        error!("cargo build failed with {status}");
        process::exit(1);
    }
}

fn xtask_build_jh7110_main(env: &Env) {
    trace!("build JH7110 main");
    let mut command = get_cargo_cmd_in(env, board_project_root(), "main", "build");
    let status = command.status().unwrap();
    trace!("cargo returned {status}");
    if !status.success() {
        error!("cargo build failed with {status}");
        process::exit(1);
    }
}

fn xtask_concat_binaries(env: &Env) {
    let dist_dir = dist_dir(env, TARGET);
    let mut bt0_file = File::options()
        .read(true)
        .open(dist_dir.join(BT0_BIN))
        .expect("open bt0 binary file");
    let mut main_file = File::options()
        .read(true)
        .open(dist_dir.join(MAIN_BIN))
        .expect("open main binary file");
    let mut fdt_file = File::options()
        .read(true)
        .open(dist_dir.join(FDT_BIN))
        .expect("open oreboot FDT file");

    let output_file_path = dist_dir.join(IMAGE_BIN);
    let mut output_file = File::options()
        .write(true)
        .create(true)
        .open(&output_file_path)
        .expect("create output binary file");

    output_file.set_len(SRAM0_SIZE).unwrap(); // FIXME: depend on storage

    let bt0_len = 32 * 1024;
    io::copy(&mut bt0_file, &mut output_file).expect("copy bt0 binary");
    output_file
        .seek(SeekFrom::Start(bt0_len))
        .expect("seek after bt0 copy");
    io::copy(&mut main_file, &mut output_file).expect("copy main binary");
    io::copy(&mut fdt_file, &mut output_file).expect("copy oreboot FDT binary");

    println!("======= DONE =======");
    println!("Output file: {:?}", &output_file_path.into_os_string());
}

fn xtask_build_dtb_image(env: &Env) {
    let dist_dir = dist_dir(env, TARGET);

    let dtb_path = dist_dir.join(BOARD_DTB);
    let dtb = fs::read(dtb_path).expect("dtb");

    let output_file_path = dist_dir.join(FDT_BIN);
    let output_file = File::options()
        .write(true)
        .create(true)
        .open(&output_file_path)
        .expect("create output binary file");

    output_file.set_len(SRAM0_SIZE).unwrap(); // FIXME: depend on storage

    let fdt = fdt::Fdt::new(&dtb).unwrap();
    let mut areas: Vec<Area> = vec![];
    areas.resize(
        16,
        Area {
            name: "",
            offset: None,
            size: 0,
            file: None,
        },
    );
    let areas = create_areas(&fdt, &mut areas);

    layout_flash(
        Path::new(&dist_dir),
        Path::new(&output_file_path),
        areas.to_vec(),
    )
    .unwrap();
}

// FIXME: factor out, rework, share!
fn board_project_root() -> std::path::PathBuf {
    project_root().join("src/mainboard/starfive/visionfive2")
}
