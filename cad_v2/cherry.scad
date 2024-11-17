module cherry(c="Indigo") {
  // $fs = 0.02;
  color(c)
  difference() {
    translate([0,0,2.5])
    cube([14, 14, 5], center=true);
    // cylinder(h=1, d=3.40, center=true);
  }
}

module cherry_1u_kc(c="Ivory") {
  color(c)
  translate([0,0,1.85])
  cube([17.5, 16.5, 3.7], center=true);
}

module cherry_1u(s="", c="Ivory", with_kc=true) {
  cherry();
  if (with_kc) {
    translate([0,0,7])
    difference() {
      cherry_1u_kc(c=c);
      translate([0,0,3.5]) text(s, size=5, halign="center", valign="center");
    }
  }
}

cherry_1u("A");
translate([20,0,0])
cherry_1u(with_kc=false);