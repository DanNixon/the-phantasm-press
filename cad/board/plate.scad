include <dimensions.scad>;

module Plate() {
    difference() {
        hull() {
            for(a = [45 / 2 : 45 : 359]) {
                rotate(a) {
                    d = (plate_width / 2) - plate_vertex_radius;
                    translate([d, 0, 0]) {
                        circle(r = plate_vertex_radius, $fn = 64);
                    }
                }
            }
        }

        for(a = [45 / 2 : 45 : 359]) {
            rotate(a) {
                translate([pillar_offset, 0, 0]) {
                    circle(d = 3.1, $fn = 16);
                }
            }
        }
    }
}

Plate();
