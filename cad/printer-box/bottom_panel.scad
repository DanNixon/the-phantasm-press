include <./dimensions.scad>;

module BottomPanel() {
  // Main
  square([internal_width, internal_depth], center=true);

  // Tabs
  for(y = bottom_panel_tab_positions) {
    translate([0, y]) {
      square([internal_width + (bottom_panel_thickness * 2), bottom_panel_tab_width], center=true);
    }
  }
}

module BottomPanel3D() {
  translate([0, 0, -bottom_panel_thickness]) {
    linear_extrude(height = bottom_panel_thickness) {
      BottomPanel();
    }
  }
}

BottomPanel();
