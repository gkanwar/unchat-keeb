projection(cut = true)
difference() {
cube([150, 150, 3], center=true);
translate([0,0,-1.5]) cylinder(h=3, r=5);
}