module pico() {
  translate([0, 0, 9])
  cube([21, 51, 1], center=true);
  header_len = 0.6+2.54*20;
  color("black")
  translate([-10.5+2.54/2, 0, 8.5/2])
  cube([2.54, header_len, 8.5], center=true);
  color("black")
  translate([10.5-2.54/2, 0, 8.5/2])
  cube([2.54, header_len, 8.5], center=true);
}

pico();