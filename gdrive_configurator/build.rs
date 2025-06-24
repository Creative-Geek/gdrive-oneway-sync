fn main() {
    // This line remains the same and compiles the .slint UI file.
    slint_build::compile("ui/app.slint").unwrap();

    // This block is only compiled on Windows.
    // It finds our .rc file and tells the linker to embed the icon.
    #[cfg(windows)]
    {
        embed_resource::compile("src/windows.rc", embed_resource::NONE);
    }
}