module locked_reg_async_mux (
    input wire clk_i,
    input wire rst_ni,
    input wire [7:0] uart_data_i,
    input wire uart_sel_i,
    input wire ctrl_data_i,
    input wire ctrl_sel_i,
    // input wire [7:0] new_data_i
    input wire new_sel_i,
    output wire [7:0] uart_data_o,
    output wire ctrl_data_o,
    output wire [7:0] new_data_o
);

    // Internal wires mimicking the $verific naming convention
    wire [7:0] \$flatten\u_uart.$verific$n72$58633 ;
    wire \$flatten\u_ctrl.$verific$n96$58619 ;
    wire [7:0] new_uart_data; ;

    // Logic for UART Data Register (Pattern: AsyncDffMuxEnable)
    // This mux creates the 'Enable' by feeding back the Q output
    assign \$flatten\u_uart.$verific$n72$58633  = uart_sel_i ? uart_data_i : uart_data_o;

    always @(posedge clk_i or negedge rst_ni) begin
        if (!rst_ni) begin
            \i_pulp_io.u_uart.reg_data_q  <= 8'h00;
        end else begin
            \i_pulp_io.u_uart.reg_data_q  <= \$flatten\u_uart.$verific$n72$58633 ;
        end
    end

    assign uart_data_o = \i_pulp_io.u_uart.reg_data_q ;

    // Logic for Control Register (Pattern: AsyncDffMuxEnable)
    // Separate logic that the tool is likely misidentifying
    assign \$flatten\u_ctrl.$verific$n96$58619  = ctrl_sel_i ? ctrl_data_i : ctrl_data_o;

    always @(posedge clk_i or negedge rst_ni) begin
        if (!rst_ni) begin
            \i_pulp_io.u_ctrl.reg_bit_q  <= 1'b0;
        end else begin
            \i_pulp_io.u_ctrl.reg_bit_q  <= \$flatten\u_ctrl.$verific$n96$58619 ;
        end
    end

    assign ctrl_data_o = \i_pulp_io.u_ctrl.reg_bit_q ;

    // Logic for Control Register (Pattern: AsyncDffMuxEnable)
    // Separate logic that the tool is likely misidentifying
    assign new_uart_data  = new_sel_i ? 8'b01101001 : new_data_o;

    always @(posedge clk_i or negedge rst_ni) begin
        if (!rst_ni) begin
            new_data_reg  <= 8'h00;
        end else begin
            new_data_reg  <= new_uart_data;
        end
    end

    assign new_data_o = new_data_reg ;
    // Escaped identifiers to match netlist style
    reg [7:0] \i_pulp_io.u_uart.reg_data_q ;
    reg \i_pulp_io.u_ctrl.reg_bit_q ;
    reg [7:0] new_data_reg ;

endmodule