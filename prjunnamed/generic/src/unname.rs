use prjunnamed_netlist::{Cell, Design};

pub fn unname(design: &mut Design) {
    for cell_ref in design.iter_cells() {
        if let Cell::Name(name, value) = &*cell_ref.get() {
            cell_ref.replace(Cell::Debug(name.clone(), value.clone()));
        }
    }
    design.apply();
}
