internal_width = 148;
internal_depth = 195;
internal_front_height = 114;
internal_rear_height = 156;

spacer_positions = [
  [95, 70],
  [95, 6],
  [-91, 30],
];

side_panel_thickness = 6;

bottom_panel_thickness = 6;
bottom_panel_tab_width = 30;
bottom_panel_tab_positions = [-75, 0, 75];

top_panel_thickness = 1.6;
top_panel_tab_width = 20;
top_panel_tab_positions = [-80, -27, 27, 80];

front_panel_thickness = 1.6;
front_panel_tab_width = 20;
front_panel_tab_positions = [-40, 0, 40];

function top_panel_angle() = atan((internal_rear_height - internal_front_height) / internal_depth);
function top_panel_height() = sqrt(pow(internal_depth, 2) + pow(internal_rear_height - internal_front_height, 2));
function top_panel_assembly_height() = (internal_front_height + internal_rear_height) / 2;
