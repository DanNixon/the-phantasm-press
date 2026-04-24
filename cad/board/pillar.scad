include <dimensions.scad>;

module Pillar() {
    difference() {
        // Profile
        hull() {
            circle(d = 15);
            for(a = [-50, 50]) {
                rotate(a) {
                    translate([10, 0, 0]) {
                        circle(r = 2.5, $fn = 32);
                    }
                }
            }
        }

        // Mounting hole (for M3 brass insert)
        circle(d = 4.5, $fn = 8);

        // Side panel cutout
        for(a = [0, 180]) {
            rotate([a, 0, 0]) {
                rotate(45 / 2) {
                    translate([6.25, 5, 0]) {
                        square([side_panel_thickness + 0.2, 8], center = true);
                    }
                }
            }
        }
    }
}

module Pillar3D() {
    linear_extrude(internal_height) {
        Pillar();
    }
}

Pillar3D();
