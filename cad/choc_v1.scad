module choc_v1(c="Indigo") {
  // $fs = 0.02;
  color(c)
  difference() {
    translate([0,0,2.5])
    cube([15, 15, 5], center=true);
    // cylinder(h=1, d=3.40, center=true);
  }
}

module choc_1u_kc(c="Ivory") {
  color(c)
  translate([0,0,1.85])
  cube([17.5, 16.5, 3.7], center=true);
}

module choc_1p5u_kc(c="Ivory") {
  color(c)
  translate([0,0,1.85])
  cube([26.5, 16.5, 3.7], center=true);
}

module choc_1p5uR_kc(c="Ivory") {
  color(c)
  translate([0,0,1.85])
  cube([16.5, 26.5, 3.7], center=true);
}

module choc_1u(s="", c="Ivory", with_kc=true) {
  choc_v1();
  if (with_kc) {
    translate([0,0,7])
    difference() {
      choc_1u_kc(c=c);
      translate([0,0,3.5]) text(s, size=5, halign="center", valign="center");
    }
  }
}

module choc_1p5u(c="Ivory", with_kc=true) {
  choc_v1();
  if (with_kc) {
    translate([0,0,7])
    choc_1p5u_kc(c=c);
  }
}

module choc_1p5uR(c="Ivory", with_kc=true) {
  choc_v1();
  if (with_kc) {
    translate([0,0,7])
    choc_1p5uR_kc(c=c);
  }
}

choc_1u("A");
translate([20,0,0])
choc_1u(with_kc=false);