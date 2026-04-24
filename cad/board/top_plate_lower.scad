include <dimensions.scad>;
use <plate.scad>;

module TopPlateLower() {
    difference() {
        Plate();

        // Button mounting hole
        circle(d = 16, $fn = 16);

        // Card reader PCB mounting holes
        for(a = [0 : 90 : 359]) {
            rotate(a) {
                for(h = card_reader_mounting_holes) {
                    translate([0, card_reader_offset] + h) {
                        circle(d = 2, $fn = 16);
                    }
                }
            }
        }
    }
}

module TopPlateLower3D() {
    linear_extrude(height = top_plate_thickness) {
        TopPlateLower();
    }
}

TopPlateLower();
