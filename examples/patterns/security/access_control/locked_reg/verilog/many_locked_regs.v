// many_locked_regs.v
// ------------------------------------------------------------
// Example top‑level module that instantiates 2× each locked‑
// register type (total 8 instances).  The RTLIL files are
// converted to Verilog (see the “packing RTLIL” section below)
// and then included with `include` statements.
// ------------------------------------------------------------

`timescale 1ns/1ps

// ------------------------------------------------------------------
// Include the generated Verilog versions of the RTLIL modules.
// The include paths assume the generated files are placed next to
// this file (or adjust the relative path as needed).
// ------------------------------------------------------------------

// #### NOTE ####
// The en modules do not work, they parse as muxes inside of yosys

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