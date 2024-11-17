use <layout.scad>;

/*module pcb_outline() {
  projection() pcb();
}*/

module switches_outline() {
  projection(cut=true) switches(with_kc=false);
}

pcb_footprint();
// switches_outline();
// top_plate_outline();
// bottom_plate_outline();