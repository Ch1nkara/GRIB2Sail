slint::slint! {
    import { HorizontalBox, LineEdit, ScrollView, VerticalBox } from "std-widgets.slint";

    export component MainWindow inherits Window {
        in-out property <string> model: "";
        in-out property <string> component_name: "";
        in-out property <string> step: "";
        in-out property <string> days: "";
        in-out property <string> lat: "";
        in-out property <string> lon: "";
        in-out property <string> outdir: "";

        preferred-width: 760px;
        preferred-height: 560px;
        min-width: 360px;
        min-height: 480px;
        title: "Grib2Sail";

        ScrollView {
            width: 100%;
            height: 100%;

            VerticalBox {
                padding: 16px;
                spacing: 12px;

                Text {
                    text: "Grib2Sail";
                    font-size: 28px;
                }

                Text {
                    text: "Download parameters";
                    opacity: 0.7;
                }

                HorizontalBox {
                    spacing: 12px;
                    Text { width: 120px; text: "Model"; vertical-alignment: center; }
                    LineEdit { text <=> root.model; placeholder-text: "gfs025"; horizontal-stretch: 1; }
                }

                HorizontalBox {
                    spacing: 12px;
                    Text { width: 120px; text: "Component"; vertical-alignment: center; }
                    LineEdit { text <=> root.component_name; placeholder-text: "meteofrance"; horizontal-stretch: 1; }
                }

                HorizontalBox {
                    spacing: 12px;
                    Text { width: 120px; text: "Step"; vertical-alignment: center; }
                    LineEdit { text <=> root.step; placeholder-text: "3h"; horizontal-stretch: 1; }
                }

                HorizontalBox {
                    spacing: 12px;
                    Text { width: 120px; text: "Days"; vertical-alignment: center; }
                    LineEdit { text <=> root.days; placeholder-text: "3"; horizontal-stretch: 1; }
                }

                HorizontalBox {
                    spacing: 12px;
                    Text { width: 120px; text: "Lat"; vertical-alignment: center; }
                    LineEdit { text <=> root.lat; placeholder-text: "48.8566"; horizontal-stretch: 1; }
                }

                HorizontalBox {
                    spacing: 12px;
                    Text { width: 120px; text: "Lon"; vertical-alignment: center; }
                    LineEdit { text <=> root.lon; placeholder-text: "2.3522"; horizontal-stretch: 1; }
                }

                HorizontalBox {
                    spacing: 12px;
                    Text { width: 120px; text: "Outdir"; vertical-alignment: center; }
                    LineEdit { text <=> root.outdir; placeholder-text: "./out"; horizontal-stretch: 1; }
                }
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    ui.run()
}
