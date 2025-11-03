module cwe1280_fixed_tb;

reg [2:0] usr_id;
reg [7:0] data_in;
reg clk, rst_n;
wire [7:0] data_out;
integer cycle;

cwe1280_fixed dut (
    .data_out(data_out),
    .usr_id(usr_id),
    .data_in(data_in),
    .clk(clk),
    .rst_n(rst_n)
);

// Clock generation
initial begin
    clk = 0;
    forever #5 clk = ~clk;
end

// Test sequence
initial begin
    $dumpfile("cwe1280_fixed.vcd");
    $dumpvars(0, cwe1280_fixed_tb);
    $display("=== CWE-1280 Fixed Version Test ===");
    
    // Initialize
    cycle = 0;
    usr_id = 0;
    data_in = 0;
    rst_n = 0;
    
    // Reset
    #10;
    rst_n = 1;
    #10;
    
    // Cycle 1: Authorized user (ID=4) writes 0xAB
    usr_id = 3'h4;
    data_in = 8'hAB;
    #10;
    cycle = cycle + 1;
    $display("Cycle %0d - User ID=%0d (authorized) writes 0x%02h: data_out=0x%02h", 
             cycle, usr_id, data_in, data_out);
    
    // Cycle 2: Unauthorized user (ID=3) tries to write 0xCD
    usr_id = 3'h3;
    data_in = 8'hCD;
    #10;
    cycle = cycle + 1;
    $display("Cycle %0d - User ID=%0d (UNAUTHORIZED) writes 0x%02h: data_out=0x%02h", 
             cycle, usr_id, data_in, data_out);
    
    if (data_out == 8'hAB) begin
        $display("*** SECURE: Unauthorized user blocked, data unchanged ***");
    end else begin
        $display("*** VULNERABILITY: Unauthorized access occurred! ***");
    end
    
    // Cycle 3: Unauthorized user tries again
    usr_id = 3'h3;
    data_in = 8'hEF;
    #10;
    cycle = cycle + 1;
    $display("Cycle %0d - User ID=%0d (UNAUTHORIZED) writes 0x%02h: data_out=0x%02h", 
             cycle, usr_id, data_in, data_out);
    
    #20;
    $finish;
end

endmodule