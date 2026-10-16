slint::slint! {
    export component MainWindow inherits Window {
        width: 360px;
        height: 180px;
        title: "Grib2Sail";

        Text {
            width: parent.width;
            height: parent.height;
            text: "Hello, world!";
            horizontal-alignment: center;
            vertical-alignment: center;
            font-size: 24px;
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    ui.run()
}
