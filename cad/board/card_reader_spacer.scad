include <dimensions.scad>;

module CardReaderSpacer() {
    difference() {
        hull() {
            for(h = card_reader_mounting_holes) {
                translate(h) {
                    circle(d = 8, $fn = 16);
                }
            }
        }

        for(h = card_reader_mounting_holes) {
            translate(h) {
                circle(d = 3.2, $fn = 16);
            }
        }
    }
}

module CardReaderSpacer3D() {
    linear_extrude(card_reader_spacer_thickness) {
        CardReaderSpacer();
    }
}

CardReaderSpacer3D();
