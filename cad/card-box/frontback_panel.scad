include <./dimensions.scad>;

width = internal[0] + (side_panel_thickness * 2) + 8;
height_offset = bottom_panel_thickness + 4;
height = internal[2] + height_offset;

module FrontBackPanel() {
  translate([0, -height_offset / 2]) {
    difference() {
      // Main
      minkowski() {
        d = 8;
        square([width - d, height - d], center=true);
        circle(d = d, $fn = 32);
      }

      // Side panel tab slots
      dx = (internal[0] + side_panel_thickness) / 2;
      for(x = [-dx, dx]) {
        for(y = side_panel_tab_positions) {
          translate([x, y + (height_offset / 2)]) {
            square([side_panel_thickness, side_panel_tab_width], center=true);
          }
        }
      }

      // Bottom panel tab slots
      for(x = bottom_panel_tab_positions) {
        translate([x, (-internal[2] + height_offset - bottom_panel_thickness) / 2]) {
          square([bottom_panel_tab_width, bottom_panel_thickness], center=true);
        }
      }
    }
  }
}

module FrontBackPanel3D() {
  linear_extrude(height = frontback_panel_thickness) {
    FrontBackPanel();
  }
}

FrontBackPanel();
