use crate::util::{
    compile_board_dt, dist_dir, find_binutils_prefix_or_fail, get_cargo_cmd_in, objcopy,
    project_root,
};
use crate::{layout_flash, Cli, Commands, Env};
// use fdt;
use log::{error, info, trace, warn};
use std::{
    fs::{self, File},
    io::{self, Seek, SeekFrom},
    path::Path,
    process,
};

extern crate layoutflash;
use layoutflash::areas::{create_areas, Area};

use super::visionfive2_hdr::spl_create_hdr;

const HEADER_SIZE: usize = 0x400;

const ARCH: &str = "riscv64";
const TARGET: &str = "riscv64imac-unknown-none-elf";

const BT0_BIN: &str = "starfive-visionfive2-bt0.bin";
const BT0_ELF: &str = "starfive-visionfive2-bt0";

const MAIN_BIN: &str = "starfive-visionfive2-main.bin";
const MAIN_ELF: &str = "starfive-visionfive2-main";

const BOARD_DTFS: &str = "starfive-visionfive2-board.dtb";

const DTFS_IMAGE: &str = "starfive-visionfive2-dtfs.bin";

const IMAGE: &str = "starfive-visionfive2.bin";

pub(crate) fn execute_command(args: &Cli, features: Vec<String>) {
    match args.command {
        Commands::Make => {
            info!("building VisionFive2");
            // Get binutils first so we can fail early
            let binutils_prefix = &find_binutils_prefix_or_fail(ARCH);
            // Build the stages - should we parallelize this?
            xtask_build_jh7110_bt0(&args.env, &features);
            xtask_build_jh7110_main(&args.env);

            objcopy(&args.env, binutils_prefix, TARGET, ARCH, BT0_ELF, BT0_BIN);
            objcopy(&args.env, binutils_prefix, TARGET, ARCH, MAIN_ELF, MAIN_BIN);
            // dtfs
            compile_board_dt(&args.env, TARGET, &board_project_root(), BOARD_DTFS);
            // final image
            xtask_build_image(&args.env);
        }
        _ => {
            error!("command {:?} not implemented", args.command);
        }
    }
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

fn xtask_build_image(env: &Env) {
    let dir = dist_dir(env, TARGET);
    let dtb_path = dir.join(BOARD_DTFS);
    let dtb = fs::read(dtb_path).expect("dtb");

    let dtfs_image_path = dir.join(DTFS_IMAGE);
    let dtfs_image = File::options()
        .write(true)
        .create(true)
        .open(&dtfs_image_path)
        .expect("create output binary file");

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

    layout_flash(Path::new(&dir), Path::new(&dtfs_image_path), areas.to_vec()).unwrap();

    // TODO: how else do we do layoutflash + header?
    trace!("add header to {dtfs_image_path:?}");
    let dat = fs::read(dtfs_image_path).expect("DTFS image");
    let out = spl_create_hdr(dat[HEADER_SIZE..].to_vec());
    let out_path = dir.join(IMAGE);
    fs::write(out_path.clone(), out).expect("writing final image");

    println!("======= DONE =======");
    println!("Output file: {:?}", &out_path.into_os_string());
}

// FIXME: factor out, rework, share!
fn board_project_root() -> std::path::PathBuf {
    project_root().join("src/mainboard/starfive/visionfive2")
}
