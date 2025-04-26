#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    // Add ths icon to the .exe file
    // This can only run on windows
    let mut res = winres::WindowsResource::new();
    res.set_icon("resources/icons/128x128@2x.ico");
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {}
