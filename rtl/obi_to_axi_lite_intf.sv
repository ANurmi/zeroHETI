module obi_to_axi_lite_intf #(
    parameter int unsigned AxiAddrWidth = 32,
    parameter int unsigned AxiDataWidth = 32,
    parameter int unsigned AxiUserWidth = 0
) (
    input logic               clk_i,
    input logic               rst_ni,
          OBI_BUS.Subordinate obi_sbr,
          AXI_LITE.Master     axi_mgr
);

    typedef enum logic [1:0] {
        IDLE,
        WRITE_RESP,
        READ_RESP
    } state_t;

    state_t state_q, state_d;

    logic [AxiAddrWidth-1:0] addr_q;
    logic [AxiDataWidth-1:0] wdata_q;
    logic [AxiDataWidth/8-1:0] strb_q;

    always_comb begin
        axi_mgr.aw_valid = 1'b0;
        axi_mgr.aw_addr  = addr_q;

        axi_mgr.w_valid  = 1'b0;
        axi_mgr.w_data   = wdata_q;
        axi_mgr.w_strb   = strb_q;

        axi_mgr.b_ready  = 1'b0;

        axi_mgr.ar_valid = 1'b0;
        axi_mgr.ar_addr  = addr_q;

        axi_mgr.r_ready  = 1'b0;

        obi_sbr.gnt      = 1'b0;
        obi_sbr.rvalid   = 1'b0;
        obi_sbr.rdata    = '0;
        obi_sbr.err      = 1'b0;

        state_d = state_q;

        case (state_q)

            IDLE: begin
                obi_sbr.gnt = obi_sbr.req;

                if (obi_sbr.req) begin

                    if (obi_sbr.we) begin
                        axi_mgr.aw_valid = 1'b1;
                        axi_mgr.w_valid  = 1'b1;

                        if (axi_mgr.aw_ready && axi_mgr.w_ready) begin
                            state_d = WRITE_RESP;
                        end
                    end
                    else begin
                        axi_mgr.ar_valid = 1'b1;

                        if (axi_mgr.ar_ready) begin
                            state_d = READ_RESP;
                        end
                    end
                end
            end

            WRITE_RESP: begin
                axi_mgr.b_ready = 1'b1;

                if (axi_mgr.b_valid) begin
                    obi_sbr.rvalid = 1'b1;
                    obi_sbr.err    = axi_mgr.b_resp[1];
                    state_d        = IDLE;
                end
            end

            READ_RESP: begin
                axi_mgr.r_ready = 1'b1;

                if (axi_mgr.r_valid) begin
                    obi_sbr.rvalid = 1'b1;
                    obi_sbr.rdata  = axi_mgr.r_data;
                    obi_sbr.err    = axi_mgr.r_resp[1];
                    state_d        = IDLE;
                end
            end

            default: begin
                state_d = IDLE;
            end
        endcase
    end

    always_ff @(posedge clk_i or negedge rst_ni) begin
        if (!rst_ni) begin
            state_q <= IDLE;
            addr_q  <= '0;
            wdata_q <= '0;
            strb_q  <= '0;
        end
        else begin
            state_q <= state_d;

            if (state_q == IDLE && obi_sbr.req) begin
                addr_q  <= obi_sbr.addr;
                wdata_q <= obi_sbr.wdata;
                strb_q  <= obi_sbr.be;
            end
        end
    end

endmodule
