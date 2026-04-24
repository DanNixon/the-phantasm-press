include <dimensions.scad>;
use <plate.scad>;

module BottomPlate() {
    difference() {
        Plate();

        // Side panel slot cutouts
        for(a = [0 : 45 : 359]) {
            rotate(a) {
                translate([0, side_panel_offset, 0]) {
                    for(x = [-side_panel_tab_offset / 2, side_panel_tab_offset / 2]) {
                        translate([x, 0, 0]) {
                            square([side_panel_tab_width + 0.05, side_panel_thickness + 0.05], center = true);
                        }
                    }
                }
            }
        }

        // Cable retention cable tie cutouts
        for(x = [-5, 5]) {
            translate([x, 110]) {
                minkowski() {
                    square([2.5, 5], center = true);
                    circle(d = 2.5, $fn = 32);
                }
            }
        }

        // Control board mounting holes
        for(p = control_board_mounting_holes) {
            translate([0, control_board_offset] + p) {
                circle(d = 3.2, $fn = 16);
            }
        }

        // LED ring mount holes
        for(a = [45 : 90 : 359]) {
            rotate(a) {
                translate([0, led_ring_mount_offset, 0.1]) {
                    circle(d = 3.2, $fn = 16);
                }
            }
        }
    }
}

module BottomPlate3D() {
    linear_extrude(height = bottom_plate_thickness) {
        BottomPlate();
    }
}

BottomPlate();
