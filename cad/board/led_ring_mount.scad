include <dimensions.scad>;

module LedRingMount3D() {
    r1 = 142 / 2;
    r2 = 162 / 2;
    w = r2 - r1;
    h = 10;
    a1 = 40;

    difference() {
        translate([0, -r1 - (w / 2), 0]) {
            difference() {
                // Main body
                rotate(90 + (-a1 / 2)) {
                    rotate_extrude(angle = a1, $fn = 64) {
                        translate([r1, 0]) {
                            square([w, h]);
                        }
                    }
                }

                // LED mount pin holes
                for(a = [-12, 12]) {
                    rotate(a) {
                        translate([0, r1 + (w / 2), h / 2]) {
                            cube([8, w + 2, 3], center = true);
                        }
                    }
                }
            }
        }

        // Mounting hole (for M3 brass insert)
        translate([0, 0, -0.01]) {
            cylinder(h = h + 0.02, d = 4.5, $fn = 8);
        }
    }
}

LedRingMount3D();
