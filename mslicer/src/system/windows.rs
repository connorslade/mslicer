use std::{env, ffi::c_void, fs, iter, path::Path, process, ptr};

use anyhow::{Context, Ok, Result};
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use windows_registry::LOCAL_MACHINE;

use crate::{app::config::Config, system::arguments::Args};

const POPUP_MESSAGE: &str = "mslicer can be installed like a traditional application or run portably. Installing will add it to the start menu and set up file associations.\n\nWhat would you like to do?";

const INSTALL_PATH: &str = r"C:\Program Files\mslicer";
const START_MENU_PATH: &str = r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs";
const EXECUTEABLE: &str = "mslicer.exe";

pub fn check_install(config: &mut Config, args: &Args) -> Result<()> {
    if args.install {
        install()?;
        return Ok(());
    }

    let install_exe = Path::new(INSTALL_PATH).join(EXECUTEABLE);
    let exe = env::current_exe()?;
    if !config.portable && exe != install_exe {
        // Asks the user if they want to install or just run, returning if they want
        // to run and relaunching with `--install` as admin otherwise.
        ask_launch_type()?;

        // If we return from that call, that means we are running portably.
        // Store this prefrence.
        config.portable = true;
    }

    Ok(())
}

fn ask_launch_type() -> Result<()> {
    let res = MessageDialog::new()
        .set_title("Install mslicer?")
        .set_description(POPUP_MESSAGE)
        .set_level(MessageLevel::Info)
        .set_buttons(MessageButtons::OkCancelCustom(
            "Install".to_string(),
            "Run Portably".to_string(),
        ))
        .show();
    println!("{res:?}");

    if let MessageDialogResult::Custom(x) = res
        && x == "Run Portably"
    {
        return Ok(());
    }

    launch_install()?;
    process::exit(0)
}

pub fn launch_install() -> Result<()> {
    let exe = env::current_exe()?;
    launch(&exe, "--install", true).context("Failed to relaunch as admin")?;
    Ok(())
}

fn install() -> Result<()> {
    let install_path = Path::new(INSTALL_PATH);
    let install_exe = install_path.join(EXECUTEABLE);
    let exe = std::env::current_exe()?;

    fs::create_dir_all(install_path).unwrap();
    fs::copy(&exe, &install_exe).unwrap();
    fs::write(
        Path::new(START_MENU_PATH).join("mslicer.lnk"),
        include_bytes!("../../../dist/windows/mslicer.lnk"),
    )?;

    LOCAL_MACHINE
        .create(r"Software\Classes\.mslicer")?
        .set_string("", "mslicer")?;
    LOCAL_MACHINE
        .create(r"Software\Classes\mslicer")?
        .set_string("", "mslicer Project")?;
    for format in [".mslicer", ".stl", ".obj", ".goo", ".ctb", ".nanodlp"] {
        LOCAL_MACHINE
            .create(format!(r"Software\Classes\{format}\OpenWithProgIds"))?
            .set_string("mslicer", "")?;
    }

    // todo: make a mslicer document icon
    let exe = install_exe.as_os_str().to_string_lossy();
    LOCAL_MACHINE
        .create(r"Software\Classes\mslicer\DefaultIcon")?
        .set_string("", format!(r#""{exe}",0"#))?;
    LOCAL_MACHINE
        .create(r"Software\Classes\mslicer\shell\open\command")?
        .set_string("", format!(r#""{exe}" "%1""#))?;

    unsafe {
        SHChangeNotify(0x0800_0000, 0x0000_1000, ptr::null(), ptr::null());
    }

    // Relaunch with standard permissions.
    launch(&install_exe, "", false).context("Failed to launch")?;

    process::exit(0);
}

fn launch(file: &Path, params: &str, admin: bool) -> Option<()> {
    let operation = wide(["open", "runas"][admin as usize]);
    let file = wide(&file.to_string_lossy());
    let parameters = wide(params);

    let result = unsafe {
        ShellExecuteW(
            0,
            operation.as_ptr(),
            file.as_ptr(),
            parameters.as_ptr(),
            ptr::null(),
            1,
        )
    };

    (result > 32).then_some(())
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn ShellExecuteW(
        hwnd: usize,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show_cmd: i32,
    ) -> i32;

    fn SHChangeNotify(event: i32, flags: u32, item1: *const c_void, item2: *const c_void);
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(iter::once(0)).collect()
}
