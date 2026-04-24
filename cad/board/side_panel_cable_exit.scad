use <side_panel.scad>;
include <dimensions.scad>;

module SidePanelCableExit() {
    difference() {
        SidePanel();

        hull() {
            for(x = [-4, 4]) {
                translate([x, 6]) {
                    circle(d = 6, $fn = 32);
                }
            }
        }
    }
}

module SidePanelCableExit3D() {
    linear_extrude(height = side_panel_thickness) {
        SidePanelCableExit();
    }
}

SidePanelCableExit();
