include <dimensions.scad>;

module SidePanel() {
    translate([-side_panel_width / 2, 0]) {
        square([side_panel_width, internal_height]);
    }

    for(x = [-side_panel_tab_offset / 2, side_panel_tab_offset / 2]) {
        translate([x - side_panel_tab_width / 2, -bottom_plate_thickness]) {
            square([side_panel_tab_width, bottom_plate_thickness]);
        }
    }
}

module SidePanel3D() {
    linear_extrude(height = side_panel_thickness) {
        SidePanel();
    }
}

SidePanel();
