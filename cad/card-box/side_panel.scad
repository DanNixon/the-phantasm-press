include <./dimensions.scad>;

width = internal[2];
height = internal[1];

module SidePanel() {
  // Main
  square([width, height], center=true);

  // Tabs
  for(x = side_panel_tab_positions) {
    translate([x, 0]) {
      square([side_panel_tab_width, height + (frontback_panel_thickness * 2)], center=true);
    }
  }
}

module SidePanel3D() {
  linear_extrude(height = side_panel_thickness) {
    SidePanel();
  }
}

SidePanel();
