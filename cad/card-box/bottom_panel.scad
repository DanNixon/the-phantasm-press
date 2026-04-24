include <./dimensions.scad>;

width = internal[0] + (side_panel_thickness * 2);
height = internal[1];

module BottomPanel() {
  // Main
  square([width, height], center=true);

  // Tabs
  for(x = bottom_panel_tab_positions) {
    translate([x, 0]) {
      square([bottom_panel_tab_width, height + (frontback_panel_thickness * 2)], center=true);
    }
  }
}

module BottomPanel3D() {
  linear_extrude(height = bottom_panel_thickness) {
    BottomPanel();
  }
}

BottomPanel();
