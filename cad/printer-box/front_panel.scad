include <./dimensions.scad>;

module FrontPanel() {
  // Main
  offset_y = -0.6;
  translate([0, offset_y / 2]) {
    square([internal_width, internal_front_height + bottom_panel_thickness + offset_y], center=true);
  }

  // Tabs
  for(y = front_panel_tab_positions) {
    translate([0, y]) {
      square([internal_width + (side_panel_thickness * 2), front_panel_tab_width], center=true);
    }
  }
}

module FrontPanel3D() {
  rotate([90, 0, 0]) {
    linear_extrude(height = front_panel_thickness) {
      FrontPanel();
    }
  }
}

FrontPanel();
