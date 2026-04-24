include <./dimensions.scad>;

module TopPanel() {
  difference() {
    union() {
      // Main
      offset_y = 2.2;
      translate([0, -offset_y / 2]) {
        square([internal_width, top_panel_height() + offset_y], center=true);
      }

      // Tabs
      for(y = top_panel_tab_positions) {
        translate([0, y]) {
          square([internal_width + (side_panel_thickness * 2), top_panel_tab_width], center=true);
        }
      }
    }

    // Printer output slot cutout
    translate([0, (-top_panel_height() / 2) + 49]) {
      square([95, 33], center=true);
    }
  }
}

module TopPanel3D() {
  linear_extrude(height = top_panel_thickness) {
    TopPanel();
  }
}

TopPanel();
