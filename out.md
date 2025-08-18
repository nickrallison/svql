## Examples
### cwes
#### examples/cwes/locked_register_pat.v
```verilog
module locked_register_example
(
input [15:0] data_in,
input clk,
input resetn,
input write,
input not_lock_status,
input lock_override,
output reg [15:0] data_out
);

always @(posedge clk or negedge resetn)
    if (~resetn) // Register is reset resetn
    begin
        data_out <= 16'h0000;
    end
    else if (write & (not_lock_status | lock_override))
    begin
        data_out <= data_in;
    end
    else if (~write)
    begin
        data_out <= data_out;
    end
endmodule
```

#### examples/cwes/variant1.v
```verilog
module locked_register_example
(
input [15:0] data_in,
input clk,
input resetn,
input write,
input lock,
input scan_mode,
input debug_unlocked,
output reg [15:0] data_out
);

reg lock_status;

always @(posedge clk or negedge resetn)
    if (~resetn) // Register is reset resetn
    begin
        lock_status <= 1'b0;
    end
    else if (lock)
    begin
        lock_status <= 1'b1;
    end
    else if (~lock)
    begin
        lock_status <= lock_status;
    end
    
always @(posedge clk or negedge resetn)
    if (~resetn) // Register is reset resetn
    begin
        data_out <= 16'h0000;
    end
    else if (write & (~lock_status | scan_mode | debug_unlocked) )
    begin
        data_out <= data_in;
    end
    else if (~write)
    begin
        data_out <= data_out;
    end
endmodule
```

### patterns
#### basic
##### and
###### examples/patterns/basic/and/and_gate.v
```verilog

module and_gate
(
input a,
input b,
output y
);

assign y = a & b;
endmodule
```

###### examples/patterns/basic/and/and_seq.v
```verilog

module and_seq
(
input a,
input b,
input c,
input d,
input e,
input f,
input g,
input h,
output y
);

assign y = (((((((a & b) & c) & d) & e) & f) & g) & h);
endmodule
```

###### examples/patterns/basic/and/and_tree.v
```verilog

module and_tree
(
input a,
input b,
input c,
input d,
input e,
input f,
input g,
input h,
output y
);

assign y = (((a & b) & (c & d)) & ((e & f) & (g & h)));
endmodule
```

###### examples/patterns/basic/and/many_ands.v
```verilog

module many_ands
(
input a,
input p,
input q,
input r,
input b,
output y
);

assign y = (((a & p) & q) & r) & b;

endmodule
```

###### examples/patterns/basic/and/many_ands_2.v
```verilog

module many_ands
(
input a,
input p,
input q,
input r,
input b,
output y
);

assign y = (((a & b) & p) & q) & r;

endmodule
```

##### ff
###### examples/patterns/basic/ff/and_q_double_sdffe.v
```verilog

module and_q_double_sdffe
(
input clk,
input d1,
input d2,
input reset,
output wire q,
);

reg q1;
reg q2;

always @(posedge clk) begin
    if (reset) q1 <= 1'b0;
    else q1 <= d1;
end

always @(posedge clk) begin
    if (reset) q2 <= 1'b0;
    else q2 <= d2;
end

assign q = q1 & q2;
endmodule
```

###### examples/patterns/basic/ff/comb_d_double_sdffe.v
```verilog

module comb_d_double_sdffe
(
input clk,
input d,
input reset,
output wire q_w_1,
output wire q_w_2,
);

reg q1;
reg q2;

always @(posedge clk) begin
    if (reset) q1 <= 1'b0;
    else q1 <= d;
end

always @(posedge clk) begin
    if (reset) q2 <= 1'b0;
    else q2 <= d;
end

assign q_w_1 = q1;
assign q_w_2 = q2;

endmodule
```

###### examples/patterns/basic/ff/par_double_sdffe.v
```verilog

module par_double_sdffe
(
input clk,
input d1,
input d2,
input reset,
output wire q_w_1,
output wire q_w_2,
);

reg q1;
reg q2;

always @(posedge clk) begin
    if (reset) q1 <= 1'b0;
    else q1 <= d1;
end

always @(posedge clk) begin
    if (reset) q2 <= 1'b0;
    else q2 <= d2;
end

assign q_w_1 = q1;
assign q_w_2 = q2;

endmodule
```

###### examples/patterns/basic/ff/sdffe.v
```verilog

module sdffe
(
input clk,
input d,
input reset,
output q
);

reg q1;

always @(posedge clk) begin
    if (reset) q1 <= 1'b0;
    else q1 <= d;
end

assign q = q1;

endmodule
```

###### examples/patterns/basic/ff/seq_8_sdffe.v
```verilog

module seq_8_sdffe
(
input clk,
input d,
input reset,
output wire q
);

parameter FIFO_DEPTH = 8;

reg fifo [0:FIFO_DEPTH-1];

always @(posedge clk) begin
    if (reset) fifo[0] <= 1'b0;
    else fifo[0] <= d;
end

genvar i;
for (i = 1; i < FIFO_DEPTH; i = i + 1) begin
    always @(posedge clk) begin
        if (reset) fifo[i] <= 1'b0;
        else fifo[i] <= fifo[i-1];
    end
end

assign q = fifo[FIFO_DEPTH-1];

endmodule
```

###### examples/patterns/basic/ff/seq_double_sdffe.v
```verilog

module seq_double_sdffe
(
input clk,
input d,
input reset,
output wire q
);

reg q1;
reg q2;

always @(posedge clk) begin
    if (reset) q1 <= 1'b0;
    else q1 <= d;
end

always @(posedge clk) begin
    if (reset) q2 <= 1'b0;
    else q2 <= q1;
end

assign q = q2;

endmodule
```

###### examples/patterns/basic/ff/seq_sdffe.v
```verilog

module seq_sdffe
(
input clk,
input d,
input reset,
output wire q
);

parameter FIFO_DEPTH = 8;

reg fifo [0:FIFO_DEPTH-1];

always @(posedge clk) begin
    if (reset) fifo[0] <= 1'b0;
    else fifo[0] <= d;
end

genvar i;
for (i = 1; i < FIFO_DEPTH; i = i + 1) begin
    always @(posedge clk) begin
        if (reset) fifo[i] <= 1'b0;
        else fifo[i] <= fifo[i-1];
    end
end

assign q = fifo[FIFO_DEPTH-1];

endmodule
```

##### not
###### examples/patterns/basic/not/many_nots.v
```verilog

module many_nots
(
input a,
input b,
output y1,
output y2,
);

assign y1 = ~a;
assign y2 = ~b;
endmodule
```

###### examples/patterns/basic/not/not.v
```verilog

module not_gate
(
input a,
output y
);

assign y = ~a;
endmodule
```

##### or
###### examples/patterns/basic/or/many_ors.v
```verilog

module many_ors
(
input a,
input b,
input c,
input d,
input e,
output y
);

assign y = (((a | b) | c) | d) | e;
endmodule
```

###### examples/patterns/basic/or/or.v
```verilog

module or_gate
(
input a,
input b,
output y
);

assign y = a | b;
endmodule
```

#### security
##### access_control
###### locked_reg
####### verilog
######## examples/patterns/security/access_control/locked_reg/verilog/async_en.v
```verilog
/* Generated by Yosys 0.33 (git sha1 2584903a060) */
// Yosys Still reads as a mux
module async_en(data_in, clk, resetn, write_en, data_out);
  input clk;
  wire clk;
  input [15:0] data_in;
  wire [15:0] data_in;
  output [15:0] data_out;
  reg [15:0] data_out;
  input resetn;
  wire resetn;
  input write_en;
  wire write_en;
  always @(posedge clk, negedge resetn)
    if (!resetn) data_out <= 16'h0000;
    else if (write_en) data_out <= data_in;
endmodule
```

######## examples/patterns/security/access_control/locked_reg/verilog/async_mux.v
```verilog
/* Generated by Yosys 0.33 (git sha1 2584903a060) */

module async_mux(data_in, clk, resetn, write_en, data_out);
  wire [15:0] _0_;
  input clk;
  wire clk;
  input [15:0] data_in;
  wire [15:0] data_in;
  output [15:0] data_out;
  reg [15:0] data_out;
  input resetn;
  wire resetn;
  input write_en;
  wire write_en;
  always @(posedge clk, negedge resetn)
    if (!resetn) data_out <= 16'h0000;
    else data_out <= _0_;
  assign _0_ = write_en ? data_in : data_out;
endmodule
```

######## examples/patterns/security/access_control/locked_reg/verilog/many_locked_regs.v
```verilog
// many_locked_regs.v
`timescale 1ns/1ps

// #### NOTE ####
// The en modules do not work, they parse as muxes inside of yosys, need to use .il files instead.
`include "async_en.v"
`include "async_mux.v"
`include "sync_en.v"
`include "sync_mux.v"

module many_locked_regs (
    input  wire        clk,
    input  wire        rst_n,

    // --- async_en -------------------------------------------------
    input  wire [15:0] async_en_data_in_0,
    input  wire        async_en_write_en_0,
    output wire [15:0] async_en_data_out_0,

    input  wire [15:0] async_en_data_in_1,
    input  wire        async_en_write_en_1,
    output wire [15:0] async_en_data_out_1,

    // --- async_mux ------------------------------------------------
    input  wire [15:0] async_mux_data_in_0,
    input  wire        async_mux_write_en_0,
    output wire [15:0] async_mux_data_out_0,

    input  wire [15:0] async_mux_data_in_1,
    input  wire        async_mux_write_en_1,
    output wire [15:0] async_mux_data_out_1,

    // --- sync_en --------------------------------------------------
    input  wire [15:0] sync_en_data_in_0,
    input  wire        sync_en_write_en_0,
    output wire [15:0] sync_en_data_out_0,

    input  wire [15:0] sync_en_data_in_1,
    input  wire        sync_en_write_en_1,
    output wire [15:0] sync_en_data_out_1,

    // --- sync_mux -------------------------------------------------
    input  wire [15:0] sync_mux_data_in_0,
    input  wire        sync_mux_write_en_0,
    output wire [15:0] sync_mux_data_out_0,

    input  wire [15:0] sync_mux_data_in_1,
    input  wire        sync_mux_write_en_1,
    output wire [15:0] sync_mux_data_out_1
);
    // ----------------------------------------------------------------
    // 2 × async_en
    // ----------------------------------------------------------------
    async_en u_async_en_0 (
        .clk      (clk),
        .resetn   (rst_n),
        .data_in  (async_en_data_in_0),
        .write_en (async_en_write_en_0),
        .data_out (async_en_data_out_0)
    );

    async_en u_async_en_1 (
        .clk      (clk),
        .resetn   (rst_n),
        .data_in  (async_en_data_in_1),
        .write_en (async_en_write_en_1),
        .data_out (async_en_data_out_1)
    );

    // ----------------------------------------------------------------
    // 2 × async_mux
    // ----------------------------------------------------------------
    async_mux u_async_mux_0 (
        .clk      (clk),
        .resetn   (rst_n),
        .data_in  (async_mux_data_in_0),
        .write_en (async_mux_write_en_0),
        .data_out (async_mux_data_out_0)
    );

    async_mux u_async_mux_1 (
        .clk      (clk),
        .resetn   (rst_n),
        .data_in  (async_mux_data_in_1),
        .write_en (async_mux_write_en_1),
        .data_out (async_mux_data_out_1)
    );

    // ----------------------------------------------------------------
    // 2 × sync_en
    // ----------------------------------------------------------------
    sync_en u_sync_en_0 (
        .clk      (clk),
        .resetn   (rst_n),
        .data_in  (sync_en_data_in_0),
        .write_en (sync_en_write_en_0),
        .data_out (sync_en_data_out_0)
    );

    sync_en u_sync_en_1 (
        .clk      (clk),
        .resetn   (rst_n),
        .data_in  (sync_en_data_in_1),
        .write_en (sync_en_write_en_1),
        .data_out (sync_en_data_out_1)
    );

    // ----------------------------------------------------------------
    // 2 × sync_mux
    // ----------------------------------------------------------------
    sync_mux u_sync_mux_0 (
        .clk      (clk),
        .resetn   (rst_n),
        .data_in  (sync_mux_data_in_0),
        .write_en (sync_mux_write_en_0),
        .data_out (sync_mux_data_out_0)
    );

    sync_mux u_sync_mux_1 (
        .clk      (clk),
        .resetn   (rst_n),
        .data_in  (sync_mux_data_in_1),
        .write_en (sync_mux_write_en_1),
        .data_out (sync_mux_data_out_1)
    );

endmodule
```

######## examples/patterns/security/access_control/locked_reg/verilog/sync_en.v
```verilog
/* Generated by Yosys 0.33 (git sha1 2584903a060) */
// Yosys Still reads as a mux
module sync_en(data_in, clk, resetn, write_en, data_out);
  input clk;
  wire clk;
  input [15:0] data_in;
  wire [15:0] data_in;
  output [15:0] data_out;
  reg [15:0] data_out;
  input resetn;
  wire resetn;
  input write_en;
  wire write_en;
  always @(posedge clk)
    if (!resetn) data_out <= 16'h0000;
    else if (write_en) data_out <= data_in;
endmodule
```

######## examples/patterns/security/access_control/locked_reg/verilog/sync_mux.v
```verilog
/* Generated by Yosys 0.33 (git sha1 2584903a060) */

module sync_mux(data_in, clk, resetn, write_en, data_out);
  wire [15:0] _0_;
  wire [15:0] _1_;
  input clk;
  wire clk;
  input [15:0] data_in;
  wire [15:0] data_in;
  output [15:0] data_out;
  reg [15:0] data_out;
  input resetn;
  wire resetn;
  input write_en;
  wire write_en;
  always @(posedge clk)
    data_out <= _0_;
  assign _1_ = write_en ? data_in : data_out;
  assign _0_ = resetn ? _1_ : 16'h0000;
endmodule
```

## svql_subgraph
### src
#### svql_subgraph/src/anchor.rs
```rust
use super::cell_kind::CellKind;
use super::index::{Index, NodeId};

pub(super) fn choose_anchors<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
) -> Option<(CellKind, Vec<NodeId>, Vec<NodeId>)> {
    // Count kinds in design
    let mut design_counts = Vec::new();
    for (&kind, nodes) in d_index_kind_iter(d_index) {
        design_counts.push((kind, nodes.len()));
    }

    // Candidate kinds: present in both pattern and design
    let mut candidates: Vec<(CellKind, usize)> = design_counts
        .into_iter()
        .filter(|(k, _)| !p_index.of_kind(*k).is_empty())
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Pick the rarest kind in the design
    candidates.sort_by(|a, b| a.1.cmp(&b.1));
    let anchor_kind = candidates[0].0;

    let p_anchors = p_index.of_kind(anchor_kind).to_vec();
    let d_anchors = d_index.of_kind(anchor_kind).to_vec();

    if p_anchors.is_empty() || d_anchors.is_empty() {
        return None;
    }

    Some((anchor_kind, p_anchors, d_anchors))
}

fn d_index_kind_iter<'a>(
    d_index: &'a Index<'a>,
) -> impl Iterator<Item = (&'a CellKind, &'a [super::index::NodeId])> {
    // Build a slice of tuples for iteration
    let mut kinds = Vec::new();
    for k in all_gate_kinds() {
        let nodes = d_index.of_kind(*k);
        if !nodes.is_empty() {
            kinds.push((k, nodes));
        }
    }
    kinds.into_iter()
}

fn all_gate_kinds() -> &'static [CellKind] {
    use CellKind::*;
    &[
        Buf, Not, And, Or, Xor, Mux, Adc, Aig, Eq, ULt, SLt, Shl, UShr, SShr, XShr, Mul, UDiv,
        UMod, SDivTrunc, SDivFloor, SModTrunc, SModFloor, Dff,
    ]
}

#[cfg(test)]
mod tests {
    use prjunnamed_netlist::Design;

    use crate::util::load_design_from;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = load_design_from("examples/patterns/basic/ff/sdffe.v").unwrap();
    }

    #[test]
    fn choose_anchors_some() {
        let d = &SDFFE;
        let p_index = super::Index::build(d);
        let d_index = super::Index::build(d);
        let chosen = choose_anchors(&p_index, &d_index);
        assert!(chosen.is_some());
    }
}
```

#### svql_subgraph/src/cell_kind.rs
```rust
use std::{borrow::Cow, collections::HashMap, hash::Hash};

use prjunnamed_netlist::{Cell, CellRef, Design, MetaItemRef, Net, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CellKind {
    Buf,
    Not,
    And,
    Or,
    Xor,
    Mux,
    Adc,
    Aig,
    Eq,
    ULt,
    SLt,
    Shl,
    UShr,
    SShr,
    XShr,
    Mul,
    UDiv,
    UMod,
    SDivTrunc,
    SDivFloor,
    SModTrunc,
    SModFloor,
    Match,
    Assign,
    Dff,
    Memory,
    IoBuf,
    Target,
    Other,
    Input,
    Output,
    Name,
    Debug,
}

impl CellKind {
    pub fn is_gate(&self) -> bool {
        matches!(
            self,
            CellKind::Buf
                | CellKind::Not
                | CellKind::And
                | CellKind::Or
                | CellKind::Xor
                | CellKind::Mux
                | CellKind::Adc
                | CellKind::Aig
                | CellKind::Eq
                | CellKind::ULt
                | CellKind::SLt
                | CellKind::Shl
                | CellKind::UShr
                | CellKind::SShr
                | CellKind::XShr
                | CellKind::Mul
                | CellKind::UDiv
                | CellKind::UMod
                | CellKind::SDivTrunc
                | CellKind::SDivFloor
                | CellKind::SModTrunc
                | CellKind::SModFloor
                | CellKind::Dff
        )
    }
}

impl From<&Cell> for CellKind {
    fn from(c: &Cell) -> Self {
        match c {
            Cell::Buf(..) => CellKind::Buf,
            Cell::Not(..) => CellKind::Not,
            Cell::And(..) => CellKind::And,
            Cell::Or(..) => CellKind::Or,
            Cell::Xor(..) => CellKind::Xor,
            Cell::Mux(..) => CellKind::Mux,
            Cell::Adc(..) => CellKind::Adc,
            Cell::Aig(..) => CellKind::Aig,
            Cell::Eq(..) => CellKind::Eq,
            Cell::ULt(..) => CellKind::ULt,
            Cell::SLt(..) => CellKind::SLt,
            Cell::Shl(..) => CellKind::Shl,
            Cell::UShr(..) => CellKind::UShr,
            Cell::SShr(..) => CellKind::SShr,
            Cell::XShr(..) => CellKind::XShr,
            Cell::Mul(..) => CellKind::Mul,
            Cell::UDiv(..) => CellKind::UDiv,
            Cell::UMod(..) => CellKind::UMod,
            Cell::SDivTrunc(..) => CellKind::SDivTrunc,
            Cell::SDivFloor(..) => CellKind::SDivFloor,
            Cell::SModTrunc(..) => CellKind::SModTrunc,
            Cell::SModFloor(..) => CellKind::SModFloor,
            Cell::Match(..) => CellKind::Match,
            Cell::Assign(..) => CellKind::Assign,
            Cell::Dff(..) => CellKind::Dff,
            Cell::Memory(..) => CellKind::Memory,
            Cell::IoBuf(..) => CellKind::IoBuf,
            Cell::Target(..) => CellKind::Target,
            Cell::Other(..) => CellKind::Other,
            Cell::Input(..) => CellKind::Input,
            Cell::Output(..) => CellKind::Output,
            Cell::Name(..) => CellKind::Name,
            Cell::Debug(..) => CellKind::Debug,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellWrapper<'p> {
    pub cref: CellRef<'p>,
}

impl<'p> CellWrapper<'p> {
    pub fn new(cref: CellRef<'p>) -> Self {
        CellWrapper { cref }
    }
    pub fn cref(&self) -> CellRef<'p> {
        self.cref
    }
    pub fn debug_index(&self) -> usize {
        self.cref.debug_index()
    }
    pub fn get(self) -> Cow<'p, Cell> {
        self.cref.get()
    }

    pub fn metadata(&self) -> MetaItemRef<'p> {
        self.cref.metadata()
    }

    pub fn output_len(&self) -> usize {
        self.cref.output_len()
    }

    pub fn output(&self) -> Value {
        self.cref.output()
    }

    pub fn visit(&self, f: impl FnMut(Net)) {
        self.cref.visit(f)
    }

    pub fn replace(&self, to_cell: Cell) {
        self.cref.replace(to_cell)
    }

    pub fn append_metadata(&self, metadata: MetaItemRef<'p>) {
        self.cref.append_metadata(metadata)
    }

    pub fn unalive(&self) {
        self.cref.unalive()
    }

    pub fn design(self) -> &'p Design {
        self.cref.design()
    }
}

impl std::fmt::Debug for CellWrapper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let index: usize = self.cref.debug_index();
        let metadata: MetaItemRef = self.cref.metadata();

        f.debug_struct("CellWrapper")
            .field("index", &index)
            .field("meta", &metadata)
            .field("cell", self.cref.get().as_ref())
            .finish()
    }
}

impl<'a> From<CellRef<'a>> for CellWrapper<'a> {
    fn from(cref: CellRef<'a>) -> Self {
        CellWrapper { cref }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputCell<'p> {
    pub cref: CellWrapper<'p>,
}

impl<'p> InputCell<'p> {
    pub fn name(&self) -> Option<&'p str> {
        match self.cref.cref().get() {
            Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn get_gates(&self) -> Vec<CellRef<'p>> {
        if matches!(self.cref.cref().get().as_ref(), Cell::Input(_, _)) {
            get_fanout(self.cref.cref().design(), self.cref.cref())
        } else {
            vec![]
        }
    }
}

pub(crate) fn get_fanout<'a>(design: &'a Design, cell: CellRef<'a>) -> Vec<CellRef<'a>> {
    let mut fanout: Vec<CellRef<'a>> = Vec::new();

    for dest in design.iter_cells() {
        if dest == cell {
            continue;
        }

        let mut driven_by_cell = false;
        dest.visit(|net| {
            if driven_by_cell {
                return;
            }
            if let Ok((src, _bit)) = design.find_cell(net)
                && src == cell
            {
                driven_by_cell = true;
            }
        });

        if driven_by_cell {
            fanout.push(dest);
        }
    }

    fanout
}

pub(crate) fn get_input_cells<'a>(design: &'a Design) -> Vec<InputCell<'a>> {
    design
        .iter_cells()
        .filter(|cell_ref| matches!(cell_ref.get().as_ref(), Cell::Input(_, _)))
        .map(|cref| InputCell { cref: cref.into() })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputCell<'p> {
    pub cref: CellWrapper<'p>,
}

impl<'p> OutputCell<'p> {
    pub fn name(&self) -> Option<&'p str> {
        match self.cref.cref().get() {
            Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn get_gate(&self) -> CellRef<'p> {
        let mut source: Option<CellRef<'p>> = None;
        if matches!(self.cref.cref().get().as_ref(), Cell::Output(_, _)) {
            self.cref.cref().visit(|net| {
                if let Ok((src, _bit)) = self.cref.cref().design().find_cell(net) {
                    source = Some(src);
                }
            });
        }
        source.expect("Output cell should have a driving source")
    }
}

pub(crate) fn get_output_cells<'a>(design: &'a Design) -> Vec<OutputCell<'a>> {
    design
        .iter_cells()
        .filter(|cell_ref| matches!(cell_ref.get().as_ref(), Cell::Output(_, _)))
        .map(|cref| OutputCell { cref: cref.into() })
        .collect()
}

pub(crate) fn count_cells_by_kind<'a>(
    design: &'a Design,
    filter: impl Fn(CellRef<'a>) -> bool,
) -> Vec<(CellKind, usize)> {
    let mut counts = HashMap::new();
    for cell_ref in design.iter_cells().filter(|c| filter(*c)) {
        let kind = CellKind::from(cell_ref.get().as_ref());
        *counts.entry(kind).or_insert(0) += 1;
    }
    counts.into_iter().collect::<Vec<_>>()
}

pub(crate) fn is_gate(c: &Cell) -> bool {
    CellKind::from(c).is_gate()
}

pub(crate) fn is_gate_cell_ref(c: CellRef<'_>) -> bool {
    CellKind::from(c.get().as_ref()).is_gate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_gate_kind() {
        // Gates
        for k in [
            CellKind::Buf,
            CellKind::Not,
            CellKind::And,
            CellKind::Or,
            CellKind::Xor,
            CellKind::Mux,
            CellKind::Adc,
            CellKind::Aig,
            CellKind::Eq,
            CellKind::ULt,
            CellKind::SLt,
            CellKind::Shl,
            CellKind::UShr,
            CellKind::SShr,
            CellKind::XShr,
            CellKind::Mul,
            CellKind::UDiv,
            CellKind::UMod,
            CellKind::SDivTrunc,
            CellKind::SDivFloor,
            CellKind::SModTrunc,
            CellKind::SModFloor,
            CellKind::Dff,
        ] {
            assert!(k.is_gate(), "kind {:?} must be considered a gate", k);
        }

        // Not gates
        for k in [
            CellKind::Input,
            CellKind::Output,
            CellKind::IoBuf,
            CellKind::Assign,
            CellKind::Match,
            CellKind::Target,
            CellKind::Other,
            CellKind::Name,
            CellKind::Debug,
            CellKind::Memory,
        ] {
            assert!(!k.is_gate(), "kind {:?} must NOT be considered a gate", k);
        }
    }
}
```

#### svql_subgraph/src/compat.rs
```rust
use super::index::{Index, NodeId};
use super::ports::{Source, is_commutative, normalize_commutative};
use super::state::State;

pub(super) fn cells_compatible<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    state: &State<'p, 'd>,
) -> bool {
    let pk = p_index.kind(p_id);
    let dk = d_index.kind(d_id);
    if pk != dk {
        return false;
    }

    let p_pins = &p_index.pins(p_id).inputs;
    let d_pins = &d_index.pins(d_id).inputs;
    if p_pins.len() != d_pins.len() {
        return false;
    }

    if is_commutative(pk) {
        let mut p_sorted = p_pins.clone();
        let mut d_sorted = d_pins.clone();
        normalize_commutative(&mut p_sorted);
        normalize_commutative(&mut d_sorted);
        pins_compatible_pairwise(&p_sorted, &d_sorted, p_index, d_index, state)
    } else {
        pins_compatible_pairwise(p_pins, d_pins, p_index, d_index, state)
    }
}

fn pins_compatible_pairwise<'p, 'd>(
    p_pins: &[(super::ports::PinKind, Source<'p>)],
    d_pins: &[(super::ports::PinKind, Source<'d>)],
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    state: &State<'p, 'd>,
) -> bool {
    for ((_, p_src), (_, d_src)) in p_pins.iter().zip(d_pins.iter()) {
        match (p_src, d_src) {
            (Source::Const(pc), Source::Const(dc)) => {
                if pc != dc {
                    return false;
                }
            }
            (Source::Gate(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                // If the source gate in pattern is already mapped, enforce it matches.
                if let Some(p_node) = p_index.try_cell_to_node(*p_cell)
                    && let Some(mapped_d_node) = state.mapped_to(p_node)
                {
                    if let Some(d_node) = d_index.try_cell_to_node(*d_cell) {
                        if mapped_d_node != d_node || p_bit != d_bit {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
            (Source::Io(p_cell, p_bit), Source::Io(d_cell, d_bit)) => {
                if let Some((exp_d_cell, exp_d_bit)) = state.boundary_get(*p_cell, *p_bit)
                    && (exp_d_cell != *d_cell || exp_d_bit != *d_bit)
                {
                    return false;
                }
            }
            (Source::Io(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                if let Some((exp_d_cell, exp_d_bit)) = state.boundary_get(*p_cell, *p_bit)
                    && (exp_d_cell != *d_cell || exp_d_bit != *d_bit)
                {
                    return false;
                }
            }
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use prjunnamed_netlist::Design;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/sdffe.v").unwrap();
        static ref SEQ_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/seq_double_sdffe.v").unwrap();
    }

    #[test]
    fn same_cell_kind_is_compatible_with_itself() {
        let d = &*SDFFE;
        let idx = Index::build(d);
        let st = State::<'_, '_>::new(idx.gate_count());

        for &n in idx.of_kind(crate::cell_kind::CellKind::Dff) {
            assert!(cells_compatible(n, n, &idx, &idx, &st));
        }
    }

    #[test]
    fn pattern_io_can_bind_to_design_gate() {
        let d_p = &SDFFE;
        let d_d = &SEQ_DOUBLE_SDFFE;

        let p_idx = Index::build(d_p);
        let d_idx = Index::build(d_d);
        let st = State::new(p_idx.gate_count());

        let p = p_idx.of_kind(crate::cell_kind::CellKind::Dff)[0];
        for &d in d_idx.of_kind(crate::cell_kind::CellKind::Dff) {
            assert!(
                cells_compatible(p, d, &p_idx, &d_idx, &st),
                "pattern IO D should be compatible with design DFF regardless of external driver kind"
            );
        }
    }
}
```

#### svql_subgraph/src/index.rs
```rust
use std::collections::HashMap;

use prjunnamed_netlist::Design;

use crate::cell_kind::CellWrapper;

use super::cell_kind::CellKind;
use super::ports::{CellPins, extract_pins};

pub(super) type NodeId = u32;

#[derive(Clone, Debug)]
pub(super) struct Index<'a> {
    design: &'a Design,
    nodes: Vec<CellWrapper<'a>>,
    kinds: Vec<CellKind>,
    pins: Vec<CellPins<'a>>,
    by_kind: HashMap<CellKind, Vec<NodeId>>,
    cell_to_id: HashMap<CellWrapper<'a>, NodeId>,
    gate_count: usize,
}

impl<'a> Index<'a> {
    pub(super) fn build(design: &'a Design) -> Self {
        let mut nodes: Vec<CellWrapper<'a>> = Vec::new();
        let mut kinds: Vec<CellKind> = Vec::new();
        let mut pins: Vec<CellPins<'a>> = Vec::new();
        let mut by_kind: HashMap<CellKind, Vec<NodeId>> = HashMap::new();
        let mut cell_to_id: HashMap<CellWrapper<'a>, NodeId> = HashMap::new();
        let mut gate_count = 0usize;

        for cell in design.iter_cells().map(CellWrapper::new) {
            let k = CellKind::from(cell.get().as_ref());
            if !(k.is_gate()) {
                continue;
            }
            let id = nodes.len() as NodeId;
            gate_count += 1;
            nodes.push(cell);
            kinds.push(k);
            let p = extract_pins(cell);
            pins.push(p);
            by_kind.entry(k).or_default().push(id);
            cell_to_id.insert(cell, id);
        }

        Index {
            design,
            nodes,
            kinds,
            pins,
            by_kind,
            cell_to_id,
            gate_count,
        }
    }

    pub(super) fn design(&self) -> &'a Design {
        self.design
    }

    pub(super) fn node_to_cell(&self, id: NodeId) -> CellWrapper<'a> {
        self.nodes[id as usize]
    }
    pub(super) fn kind(&self, id: NodeId) -> CellKind {
        self.kinds[id as usize]
    }
    pub(super) fn pins(&self, id: NodeId) -> &CellPins<'a> {
        &self.pins[id as usize]
    }

    pub(super) fn of_kind(&self, k: CellKind) -> &[NodeId] {
        self.by_kind.get(&k).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub(super) fn gate_count(&self) -> usize {
        self.gate_count
    }

    pub(super) fn try_cell_to_node(&self, c: CellWrapper<'a>) -> Option<NodeId> {
        self.cell_to_id.get(&c).copied()
    }

    pub(super) fn by_kind_iter(&self) -> Vec<(&CellKind, &[NodeId])> {
        self.by_kind
            .iter()
            .map(|(k, v)| (k, v.as_slice()))
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/sdffe.v").unwrap();
    }

    #[test]
    fn build_index_has_gates() {
        let d = &*SDFFE;
        let idx = Index::build(d);
        assert!(idx.gate_count() > 0);
        assert_eq!(idx.of_kind(CellKind::Dff).len() > 0, true);
    }
}
```

#### svql_subgraph/src/lib.rs
```rust
use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::Design;

pub mod cell_kind;
use cell_kind::{InputCell, OutputCell, get_input_cells, get_output_cells};

use crate::cell_kind::CellWrapper;

mod anchor;
mod compat;
mod index;
mod ports;
mod search;
mod state;
mod strategy;
pub(crate) mod util;

#[derive(Clone, Debug)]
pub struct AllSubgraphMatches<'p, 'd> {
    pub matches: Vec<SubgraphMatch<'p, 'd>>,
    pub _p_index: index::Index<'p>,
    pub _d_index: index::Index<'d>,
}

impl<'p, 'd> AllSubgraphMatches<'p, 'd> {
    pub fn len(&self) -> usize {
        self.matches.len()
    }
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, SubgraphMatch<'p, 'd>> {
        self.matches.iter()
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphMatch<'p, 'd> {
    pub cell_mapping: HashMap<CellWrapper<'p>, CellWrapper<'d>>,
    pub pat_input_cells: Vec<InputCell<'p>>,
    pub pat_output_cells: Vec<OutputCell<'p>>,
    pub boundary_src_map: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)>,

    // lookup indices
    pub input_by_name: HashMap<&'p str, CellWrapper<'p>>,
    pub output_by_name: HashMap<&'p str, CellWrapper<'p>>,
    pub out_driver_map: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)>,
}

impl<'p, 'd> SubgraphMatch<'p, 'd> {
    pub fn len(&self) -> usize {
        self.cell_mapping.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cell_mapping.is_empty()
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, CellWrapper<'p>, CellWrapper<'d>> {
        self.cell_mapping.iter()
    }

    pub fn design_source_of_input_bit(
        &self,
        name: &str,
        bit: usize,
    ) -> Option<(CellWrapper<'d>, usize)> {
        let p_in = *self.input_by_name.get(name)?;
        self.boundary_src_map.get(&(p_in, bit)).copied()
    }

    pub fn design_driver_of_output_bit(
        &self,
        name: &str,
        bit: usize,
    ) -> Option<(CellWrapper<'d>, usize)> {
        let p_out = *self.output_by_name.get(name)?;
        self.out_driver_map.get(&(p_out, bit)).copied()
    }
}

fn match_signature<'p, 'd>(m: &SubgraphMatch<'p, 'd>) -> Vec<(u8, usize, usize, usize, usize)> {
    let mut sig: Vec<(u8, usize, usize, usize, usize)> = Vec::new();

    for (p, d) in m.cell_mapping.iter() {
        sig.push((0, p.debug_index(), 0, d.debug_index(), 0));
    }

    for ((p_cell, p_bit), (d_cell, d_bit)) in m.boundary_src_map.iter() {
        sig.push((
            1,
            p_cell.debug_index(),
            *p_bit,
            d_cell.debug_index(),
            *d_bit,
        ));
    }

    sig.sort_unstable();
    sig
}

// Public API
pub fn find_subgraphs<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
) -> AllSubgraphMatches<'p, 'd> {
    let p_index = index::Index::build(pattern);
    let d_index = index::Index::build(design);

    if p_index.gate_count() == 0 || d_index.gate_count() == 0 {
        return AllSubgraphMatches {
            matches: Vec::new(),
            _p_index: p_index,
            _d_index: d_index,
        };
    }

    let Some((_anchor_kind, p_anchors, d_anchors)) = anchor::choose_anchors(&p_index, &d_index)
    else {
        return AllSubgraphMatches {
            matches: Vec::new(),
            _p_index: p_index,
            _d_index: d_index,
        };
    };

    let mut results: Vec<SubgraphMatch<'p, 'd>> = Vec::new();
    let (pat_inputs, pat_outputs) = get_pattern_io_cells(pattern);

    // Canonicalize pattern anchor to avoid multiplicity
    let p_anchor = *p_anchors.iter().min().unwrap();
    let p_anchors = vec![p_anchor];

    for &p_a in &p_anchors {
        for &d_a in &d_anchors {
            if p_index.kind(p_a) != d_index.kind(d_a) {
                continue;
            }
            let empty_state = state::State::<'p, 'd>::new(p_index.gate_count());
            if !compat::cells_compatible(p_a, d_a, &p_index, &d_index, &empty_state) {
                continue;
            }

            let mut st = state::State::new(p_index.gate_count());
            st.map(p_a, d_a);

            // Add IO boundaries implied by anchor mapping
            let added = search::add_io_boundaries_from_pair(p_a, d_a, &p_index, &d_index, &mut st);

            search::backtrack(
                &p_index,
                &d_index,
                &mut st,
                &mut results,
                &pat_inputs,
                &pat_outputs,
            );

            // Backtrack anchor boundaries
            search::remove_boundaries(added, &mut st);
            st.unmap(p_a, d_a);
        }
    }

    // Dedupe by combined signature
    let mut seen: HashSet<Vec<(u8, usize, usize, usize, usize)>> = HashSet::new();
    results.retain(|m| seen.insert(match_signature(m)));

    AllSubgraphMatches {
        matches: results,
        _p_index: p_index,
        _d_index: d_index,
    }
}

// Helper used by tests and callers
pub fn get_pattern_io_cells<'p>(pattern: &'p Design) -> (Vec<InputCell<'p>>, Vec<OutputCell<'p>>) {
    (get_input_cells(pattern), get_output_cells(pattern))
}

#[cfg(test)]
mod tests {
    use crate::util::load_design_from;

    use super::*;

    lazy_static::lazy_static! {
        static ref ASYNC_MUX: Design = crate::util::load_design_from("examples/patterns/security/access_control/locked_reg/json/async_mux.json").unwrap();
        static ref SEQ_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/seq_double_sdffe.v").unwrap();
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/sdffe.v").unwrap();
        static ref COMB_D_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/comb_d_double_sdffe.v").unwrap();
        static ref PAR_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/par_double_sdffe.v").unwrap();

    }

    #[test]
    fn smoke_io_cells() {
        let design = &ASYNC_MUX;
        let (ins, outs) = get_pattern_io_cells(&design);
        assert!(!ins.is_empty());
        assert!(!outs.is_empty());
    }

    #[test]
    fn smoke_find_subgraphs_self_sdffe() {
        let design = &SDFFE;
        let matches = find_subgraphs(design, design);
        assert!(
            !matches.is_empty(),
            "Self-match sdffe should yield at least one mapping"
        );
        for m in matches.iter() {
            assert!(!m.is_empty());
        }
    }

    #[test]
    fn smoke_seq_double_sdffe_has_at_least_one() {
        let design = &SEQ_DOUBLE_SDFFE;
        let matches = find_subgraphs(design, design);
        assert!(
            !matches.is_empty(),
            "Self-match seq_double_sdffe should yield mappings"
        );
    }

    #[test]
    fn exact_two_matches_comb_d_double_self() {
        let design = &COMB_D_DOUBLE_SDFFE;
        let matches = find_subgraphs(design, design);
        assert_eq!(
            matches.len(),
            2,
            "canonical anchor + dedupe should yield 2 mappings"
        );
    }

    #[test]
    fn exact_two_matches_sdffe_in_seq_double() {
        let pat = &SDFFE;
        let hay = &SEQ_DOUBLE_SDFFE;
        let matches = find_subgraphs(pat, hay);
        assert_eq!(
            matches.len(),
            2,
            "pattern IO should bind to gate, yielding 2 matches"
        );
    }

    #[test]
    fn dedupe_eliminates_anchor_duplicates_par_double_self() {
        let design = &PAR_DOUBLE_SDFFE;
        let matches = find_subgraphs(design, design);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn lookup_by_port_name_and_bit_sdffe_in_seq_double() {
        let pat = &SDFFE;
        let hay = &SEQ_DOUBLE_SDFFE;
        let all = find_subgraphs(pat, hay);
        assert_eq!(
            all.len(),
            2,
            "expected two sdffe matches in seq_double_sdffe"
        );

        // Every match should resolve both d (input) and q (output) via O(1) helpers
        for m in all.iter() {
            assert!(
                m.design_source_of_input_bit("d", 0).is_some(),
                "input d should have a bound design source"
            );
            assert!(
                m.design_driver_of_output_bit("q", 0).is_some(),
                "output q should have a resolved design driver"
            );
        }

        // There should exist a pair of matches where q of one drives d of the other.
        let mut found = false;
        let ms: Vec<_> = all.iter().collect();
        for m1 in &ms {
            if let Some((dq_cell, dq_bit)) = m1.design_driver_of_output_bit("q", 0) {
                let dq_net = dq_cell.output()[dq_bit];
                for m2 in &ms {
                    if let Some((sd_cell, sd_bit)) = m2.design_source_of_input_bit("d", 0) {
                        let sd_net = sd_cell.output()[sd_bit];
                        if dq_net == sd_net {
                            found = true;
                            break;
                        }
                    }
                }
            }
            if found {
                break;
            }
        }
        assert!(
            found,
            "expected to find at least one connection: q of one match drives d of another"
        );
    }

    #[test]
    fn connectivity_test() {
        let haystack_path = "examples/patterns/basic/ff/seq_8_sdffe.v";
        let haystack_design =
            load_design_from(&haystack_path).expect("Failed to read haystack design");

        let needle_path = "examples/patterns/basic/ff/sdffe.v";
        let needle_design = load_design_from(&needle_path).expect("Failed to read needle design");

        let search_results = find_subgraphs(&needle_design, &haystack_design);

        for m in search_results.iter() {
            assert!(
                m.design_source_of_input_bit("d", 0).is_some(),
                "input d should have a bound design source"
            );
            assert!(
                m.design_driver_of_output_bit("q", 0).is_some(),
                "output q should have a resolved design driver"
            );
        }

        let ms: Vec<_> = search_results.iter().collect();
        let mut matches = 0;
        for m1 in &ms {
            if let Some((dq_cell, dq_bit)) = m1.design_driver_of_output_bit("q", 0) {
                let dq_net = dq_cell.output()[dq_bit];
                for m2 in &ms {
                    if let Some((sd_cell, sd_bit)) = m2.design_source_of_input_bit("d", 0) {
                        let sd_net = sd_cell.output()[sd_bit];
                        if dq_net == sd_net {
                            matches += 1;
                        }
                    }
                }
            }
        }

        assert_eq!(
            matches, 7,
            "Expected 7 connections between d and q across matches, found {}",
            matches
        );
    }
}
```

#### svql_subgraph/src/ports.rs
```rust
use prjunnamed_netlist::Trit;

use crate::cell_kind::CellWrapper;

use super::cell_kind::{CellKind, is_gate_cell_ref};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum PinKind {
    Data(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Source<'a> {
    Gate(CellWrapper<'a>, usize),
    Io(CellWrapper<'a>, usize),
    Const(Trit),
}

#[derive(Clone, Debug)]
pub(super) struct CellPins<'a> {
    pub(super) kind: CellKind,
    pub(super) inputs: Vec<(PinKind, Source<'a>)>,
}

pub(super) fn is_commutative(kind: CellKind) -> bool {
    matches!(
        kind,
        CellKind::And | CellKind::Or | CellKind::Xor | CellKind::Eq
    )
}

pub(super) fn extract_pins<'a>(cref: CellWrapper<'a>) -> CellPins<'a> {
    let kind = CellKind::from(cref.get().as_ref());
    let mut idx = 0usize;
    let mut inputs: Vec<(PinKind, Source<'a>)> = Vec::new();
    cref.visit(|net| {
        let pin = PinKind::Data(idx);
        idx += 1;
        match cref.design().find_cell(net) {
            Ok((src, bit)) => {
                if is_gate_cell_ref(src) {
                    inputs.push((pin, Source::Gate(src.into(), bit)));
                } else {
                    inputs.push((pin, Source::Io(src.into(), bit)));
                }
            }
            Err(trit) => inputs.push((pin, Source::Const(trit))),
        }
    });
    CellPins { kind, inputs }
}

pub(super) fn normalize_commutative<'a>(inputs: &mut [(PinKind, Source<'a>)]) {
    inputs.sort_by(|a, b| stable_key(&a.1).cmp(&stable_key(&b.1)));
}

fn stable_key<'a>(s: &Source<'a>) -> (u8, usize, usize) {
    match s {
        Source::Const(t) => (0, (*t as i8 as i32) as usize, 0),
        Source::Io(c, bit) => (1, c.debug_index(), *bit),
        Source::Gate(c, bit) => (2, c.debug_index(), *bit),
    }
}

#[cfg(test)]
mod tests {
    use prjunnamed_netlist::Design;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/sdffe.v").unwrap();
    }

    #[test]
    fn can_extract_some_pins() {
        let d = &SDFFE;
        for c in d.iter_cells() {
            let _pins = extract_pins(c.into());
        }
    }

    #[test]
    fn commutative_sort_is_stable() {
        let d = &SDFFE;
        for c in d.iter_cells() {
            let mut pins1 = extract_pins(c.into()).inputs;
            let mut pins2 = extract_pins(c.into()).inputs;
            normalize_commutative(&mut pins1);
            normalize_commutative(&mut pins2);
            assert_eq!(pins1.len(), pins2.len());
            assert_eq!(pins1, pins2);
        }
    }
}
```

#### svql_subgraph/src/search.rs
```rust
use crate::cell_kind::CellWrapper;

use super::compat::cells_compatible;
use super::index::{Index, NodeId};
use super::ports::Source;
use super::state::State;
use super::strategy::choose_next;
use super::{
    SubgraphMatch,
    cell_kind::{InputCell, OutputCell},
};

pub(super) fn backtrack<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    st: &mut State<'p, 'd>,
    out: &mut Vec<SubgraphMatch<'p, 'd>>,
    pat_inputs: &[InputCell<'p>],
    pat_outputs: &[OutputCell<'p>],
) {
    if st.done() {
        out.push(st.to_subgraph_match(p_index, d_index, pat_inputs, pat_outputs));
        return;
    }

    let next_p = match choose_next(p_index, st) {
        Some(n) => n,
        None => return,
    };

    let kind = p_index.kind(next_p);
    for &d_cand in d_index.of_kind(kind) {
        if st.is_used_design(d_cand) {
            continue;
        }
        if !cells_compatible(next_p, d_cand, p_index, d_index, st) {
            continue;
        }

        st.map(next_p, d_cand);
        let added = add_io_boundaries_from_pair(next_p, d_cand, p_index, d_index, st);

        backtrack(p_index, d_index, st, out, pat_inputs, pat_outputs);

        remove_boundaries(added, st);
        st.unmap(next_p, d_cand);
    }
}

pub(super) fn add_io_boundaries_from_pair<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    st: &mut State<'p, 'd>,
) -> Vec<(CellWrapper<'p>, usize)> {
    let mut added = Vec::new();
    let p_pins = &p_index.pins(p_id).inputs;
    let d_pins = &d_index.pins(d_id).inputs;

    for ((_, p_src), (_, d_src)) in p_pins.iter().zip(d_pins.iter()) {
        match (p_src, d_src) {
            (Source::Io(p_cell, p_bit), Source::Io(d_cell, d_bit)) => {
                let key = (*p_cell, *p_bit);
                if st.boundary_insert(key, (*d_cell, *d_bit)) {
                    added.push(key);
                }
            }
            (Source::Io(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                let key = (*p_cell, *p_bit);
                if st.boundary_insert(key, (*d_cell, *d_bit)) {
                    added.push(key);
                }
            }
            _ => {}
        }
    }

    added
}

pub(super) fn remove_boundaries<'p, 'd>(
    added: Vec<(CellWrapper<'p>, usize)>,
    st: &mut State<'p, 'd>,
) {
    st.boundary_remove_keys(&added);
}

#[cfg(test)]
mod tests {

    use prjunnamed_netlist::Design;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/sdffe.v").unwrap();
    }

    #[test]
    fn backtrack_self_sdffe_produces_some() {
        let d = &SDFFE;
        let p_index = Index::build(d);
        let d_index = Index::build(d);

        let mut st = State::new(p_index.gate_count());
        let mut out = Vec::new();
        let inputs = super::super::cell_kind::get_input_cells(d);
        let outputs = super::super::cell_kind::get_output_cells(d);

        backtrack(&p_index, &d_index, &mut st, &mut out, &inputs, &outputs);
        if !out.is_empty() {
            assert!(!out[0].is_empty());
        }
    }
}
```

#### svql_subgraph/src/state.rs
```rust
use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::Cell;

use super::index::{Index, NodeId};
use crate::cell_kind::CellWrapper;

pub(super) struct State<'p, 'd> {
    mapping: HashMap<NodeId, NodeId>,
    used_d: HashSet<NodeId>,
    boundary: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)>,
    target_gate_count: usize,
}

impl<'p, 'd> State<'p, 'd> {
    pub(super) fn new(target_gate_count: usize) -> Self {
        State {
            mapping: HashMap::new(),
            used_d: HashSet::new(),
            boundary: HashMap::new(),
            target_gate_count,
        }
    }

    pub(super) fn is_mapped(&self, p: NodeId) -> bool {
        self.mapping.contains_key(&p)
    }
    pub(super) fn mapped_to(&self, p: NodeId) -> Option<NodeId> {
        self.mapping.get(&p).copied()
    }
    pub(super) fn is_used_design(&self, d: NodeId) -> bool {
        self.used_d.contains(&d)
    }

    pub(super) fn map(&mut self, p: NodeId, d: NodeId) {
        self.mapping.insert(p, d);
        self.used_d.insert(d);
    }

    pub(super) fn unmap(&mut self, p: NodeId, d: NodeId) {
        self.mapping.remove(&p);
        self.used_d.remove(&d);
    }

    pub(super) fn boundary_get(
        &self,
        p_cell: CellWrapper<'p>,
        p_bit: usize,
    ) -> Option<(CellWrapper<'d>, usize)> {
        self.boundary.get(&(p_cell, p_bit)).copied()
    }
    pub(super) fn boundary_insert(
        &mut self,
        key: (CellWrapper<'p>, usize),
        val: (CellWrapper<'d>, usize),
    ) -> bool {
        if self.boundary.contains_key(&key) {
            return false;
        }
        self.boundary.insert(key, val);
        true
    }
    pub(super) fn boundary_remove_keys(&mut self, keys: &[(CellWrapper<'p>, usize)]) {
        for k in keys {
            self.boundary.remove(k);
        }
    }

    pub(super) fn done(&self) -> bool {
        self.mapping.len() == self.target_gate_count
    }

    pub(super) fn to_subgraph_match(
        &self,
        p_index: &Index<'p>,
        d_index: &Index<'d>,
        pat_input_cells: &[super::cell_kind::InputCell<'p>],
        pat_output_cells: &[super::cell_kind::OutputCell<'p>],
    ) -> super::SubgraphMatch<'p, 'd> {
        let mut cell_mapping = HashMap::new();
        for (&p_node, &d_node) in &self.mapping {
            let p_cell = p_index.node_to_cell(p_node);
            let d_cell = d_index.node_to_cell(d_node);
            cell_mapping.insert(p_cell, d_cell);
        }

        let mut boundary_src_map = HashMap::new();
        for ((p_cell, p_bit), (d_cell, d_bit)) in &self.boundary {
            boundary_src_map.insert((*p_cell, *p_bit), (*d_cell, *d_bit));
        }

        // NEW: name maps
        let mut input_by_name = HashMap::new();
        for ic in pat_input_cells {
            if let Some(nm) = ic.name() {
                input_by_name.insert(nm, ic.cref);
            }
        }
        let mut output_by_name = HashMap::new();
        for oc in pat_output_cells {
            if let Some(nm) = oc.name() {
                output_by_name.insert(nm, oc.cref);
            }
        }

        // NEW: build (pattern Output bit) -> (design cell, bit) drivers
        let mut out_driver_map: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)> =
            HashMap::new();
        for oc in pat_output_cells {
            // Safely match the Output cell and pull its input Value
            if let Cell::Output(_, value) = oc.cref.cref().get().as_ref() {
                for (out_bit, net) in value.iter().enumerate() {
                    // Who drives this bit in the pattern?
                    if let Ok((p_src_cell_ref, p_bit)) = oc.cref.cref().design().find_cell(net) {
                        let p_src = CellWrapper::from(p_src_cell_ref);

                        // Prefer mapped gate
                        if let Some(&d_src) = cell_mapping.get(&p_src) {
                            out_driver_map.insert((oc.cref, out_bit), (d_src, p_bit));
                            continue;
                        }

                        // Fallback: boundary (IO-to-gate or IO-to-IO)
                        if let Some(&(d_cell, d_bit)) = self.boundary.get(&(p_src, p_bit)) {
                            out_driver_map.insert((oc.cref, out_bit), (d_cell, d_bit));
                        }
                        // else: constants/undef or unmapped sources -> no entry
                    }
                }
            }
        }

        super::SubgraphMatch {
            cell_mapping,
            pat_input_cells: pat_input_cells.to_vec(),
            pat_output_cells: pat_output_cells.to_vec(),
            boundary_src_map,
            input_by_name,  // NEW
            output_by_name, // NEW
            out_driver_map, // NEW
        }
    }
}

#[cfg(test)]
mod tests {
    use prjunnamed_netlist::Design;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/sdffe.v").unwrap();
    }

    #[test]
    fn state_basic_map_unmap() {
        let d = &SDFFE;
        let idx = Index::build(d);

        let mut st = State::new(idx.gate_count());
        let n = idx.of_kind(crate::cell_kind::CellKind::Dff)[0];
        st.map(n, n);
        assert!(st.is_mapped(n));
        assert!(st.is_used_design(n));
        st.unmap(n, n);
        assert!(!st.is_mapped(n));
        assert!(!st.is_used_design(n));
    }
}
```

#### svql_subgraph/src/strategy.rs
```rust
use super::index::Index;
use super::index::NodeId;
use super::ports::Source;
use super::state::State;

pub(super) fn choose_next<'p, 'd>(p_index: &'p Index<'p>, st: &State<'p, 'd>) -> Option<NodeId> {
    for p in 0..p_index.gate_count() {
        let p = p as NodeId;
        if st.is_mapped(p) {
            continue;
        }

        let pins = &p_index.pins(p).inputs;
        let mut all_resolvable = true;
        for (_, src) in pins {
            match src {
                Source::Const(_) => {}
                Source::Io(_, _) => {}
                Source::Gate(gc, _) => {
                    if let Some(g) = p_index.try_cell_to_node(*gc)
                        && !st.is_mapped(g)
                    {
                        all_resolvable = false;
                        break;
                    }
                }
            }
        }
        if all_resolvable {
            return Some(p);
        }
    }

    for p in 0..p_index.gate_count() {
        let p = p as NodeId;
        if !st.is_mapped(p) {
            return Some(p);
        }
    }
    None
}

#[cfg(test)]
mod tests {

    use prjunnamed_netlist::Design;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/sdffe.v").unwrap();
    }

    #[test]
    fn choose_next_returns_some() {
        let d = &SDFFE;
        let idx = Index::build(d);
        let st = State::new(idx.gate_count());
        assert!(choose_next(&idx, &st).is_some());
    }
}
```

#### svql_subgraph/src/util.rs
```rust
use std::{
    fs::File,
    path::{Path, PathBuf},
    process::Stdio,
};

use log::error;
use prjunnamed_netlist::Design;

#[allow(dead_code)]
pub(crate) fn load_design_from(design: &str) -> Result<Design, Box<dyn std::error::Error>> {
    let json_temp_file = tempfile::Builder::new()
        .prefix("svql_prjunnamed_")
        .suffix(".json")
        .rand_bytes(4)
        .tempfile()?;

    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let design = PathBuf::from(design);
    let design = if design.is_absolute() {
        design
    } else {
        workspace.join(design)
    };

    let design_path = DesignPath::new(design.to_path_buf()).unwrap();
    let yosys = which::which("yosys").map_err(|e| format!("Failed to find yosys binary: {}", e))?;
    let module_name = design
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            format!(
                "Failed to extract module name from design path: {}",
                design.display()
            )
        })?
        .to_string();

    let mut cmd = std::process::Command::new(yosys);
    cmd.args(get_command_args_slice(
        &design_path,
        &module_name,
        json_temp_file.path(),
    ));
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut yosys_process = cmd.spawn().expect("Failed to start yosys process");
    let exit_status = yosys_process
        .wait()
        .expect("Failed to wait for yosys process");

    if !exit_status.success() {
        let mut stderr = yosys_process
            .stderr
            .take()
            .expect("Failed to capture stderr");
        let mut stderr_buf = Vec::new();
        use std::io::Read;
        stderr
            .read_to_end(&mut stderr_buf)
            .expect("Failed to read stderr");
        let stderr_str = String::from_utf8_lossy(&stderr_buf);
        error!(
            "Yosys process failed with status: {:?}\nStderr: {}",
            exit_status, stderr_str
        );
        return Err(format!(
            "Yosys process failed with status: {:?}\nStderr: {}",
            exit_status, stderr_str
        )
        .into());
    }

    let designs = prjunnamed_yosys_json::import(None, &mut File::open(json_temp_file.path())?)?;
    assert_eq!(
        designs.len(),
        1,
        "can only convert single-module Yosys JSON to Unnamed IR"
    );
    let design = designs.into_values().next().unwrap();

    Ok(design)
}

fn get_command_args_slice(design: &DesignPath, module_name: &str, json_out: &Path) -> Vec<String> {
    let read_cmd = match design {
        DesignPath::Verilog(_) => "read_verilog",
        DesignPath::Rtlil(_) => "read_rtlil",
        DesignPath::Json(_) => "read_json",
    };

    vec![
        "-p".to_string(),
        format!("{} {}", read_cmd, design.path().display()),
        "-p".to_string(),
        format!("hierarchy -top {}", module_name),
        "-p".to_string(),
        "proc; flatten; opt_clean".to_string(),
        "-p".to_string(),
        format!("write_json {}", json_out.display()),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum DesignPath {
    Verilog(PathBuf),
    Rtlil(PathBuf),
    Json(PathBuf),
}

impl DesignPath {
    fn new(path: PathBuf) -> Result<Self, String> {
        if path.extension().and_then(|s| s.to_str()) == Some("v") {
            Ok(DesignPath::Verilog(path))
        } else if path.extension().and_then(|s| s.to_str()) == Some("il") {
            Ok(DesignPath::Rtlil(path))
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            Ok(DesignPath::Json(path))
        } else {
            Err(format!(
                "Unsupported design file extension: {:?}",
                path.extension()
            ))
        }
    }
    fn path(&self) -> &Path {
        match self {
            DesignPath::Verilog(p) => p,
            DesignPath::Rtlil(p) => p,
            DesignPath::Json(p) => p,
        }
    }
    fn exists(&self) -> bool {
        self.path().exists()
    }
}
```
