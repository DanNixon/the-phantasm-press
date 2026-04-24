include <./dimensions.scad>;

module SidePanel() {
  difference() {
    // Main
    hull() {
      for(p = [
        [-internal_depth / 2, -bottom_panel_thickness / 2],
        [-internal_depth / 2, internal_front_height],
        [internal_depth / 2, internal_rear_height],
        [internal_depth / 2, -bottom_panel_thickness / 2],
      ]) {
        translate(p) {
          circle(r = 6, $fn = 32);
        }
      }
    }

    // Spacer mounting holes
    for(p = spacer_positions) {
      translate([p[0], p[1]]) {
        circle(d = 3.2, $fn = 32);
      }
    }

    // Bottom panel tab slots
    for(x = bottom_panel_tab_positions) {
      translate([x, -bottom_panel_thickness / 2]) {
        square([bottom_panel_tab_width, side_panel_thickness], center=true);
      }
    }

    // Front panel tab slots
    for(y = front_panel_tab_positions) {
      translate([(-internal_depth - front_panel_thickness) / 2, ((internal_front_height - bottom_panel_thickness) / 2) + y]) {
        square([front_panel_thickness, front_panel_tab_width], center=true);
      }
    }

    // Top panel tab slots
    translate([0, top_panel_assembly_height()]) {
      rotate([0, 0, top_panel_angle()]) {
        for(x = top_panel_tab_positions) {
          translate([x, top_panel_thickness / 2]) {
            square([top_panel_tab_width, top_panel_thickness], center=true);
          }
        }
      }
    }
  }
}

module SidePanel3D() {
  rotate([90, 0, 90]) {
    translate([0, 0, -side_panel_thickness / 2]) {
      linear_extrude(height = side_panel_thickness) {
        SidePanel();
      }
    }
  }
}

SidePanel();
