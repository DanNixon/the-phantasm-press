include <dimensions.scad>;

module ControlBoardSpacer() {
    difference() {
        union() {
            for(h = control_board_mounting_holes) {
                translate(h) {
                    circle(d = 8, $fn = 16);
                }
            }

            polygon(points = [
                [-48 / 2, -1],
                [-48 / 2, 5],
                [48 / 2, 5],
                [48 / 2, -1],
            ]);

            polygon(points = [
                [-48 / 2, -65],
                [-48 / 2, 5],
                [-40 / 2, 5],
                [-40 / 2, -65],
            ]);

            polygon(points = [
                [48 / 2, -65],
                [48 / 2, 5],
                [40 / 2, 5],
                [40 / 2, -65],
            ]);
        }

        for(h = control_board_mounting_holes) {
            translate(h) {
                circle(d = 3.2, $fn = 16);
            }
        }
    }
}

module ControlBoardSpacer3D() {
    linear_extrude(control_board_spacer_thickness) {
        ControlBoardSpacer();
    }
}

ControlBoardSpacer();
