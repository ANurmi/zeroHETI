package rt_prof_pkg;

  typedef struct packed {
    logic [31:0] base;
    logic [31:0] start;
    logic [31:0] stop;
    logic [31:0] prescaler;
    logic [31:0] loadfactor;
    logic [31:0] seed;
    logic [31:0] period;
  } sim_addr_t;

  typedef struct packed {
    logic [31:0] period;
    logic [31:0] deadline;
    logic [31:0] ack;
  } task_addr_t;

  typedef struct packed {
    bit          active;
    bit          frame_active;
    bit          addr_valid;
    bit          is_write;
    logic [7:0]  data;
    int unsigned bitcount;
  } i2c_transaction_t;

  localparam sim_addr_t SimMap = '{
      base      : 32'h0100_0000,
      start     : 32'h0100_0000 + 0,
      stop      : 32'h0100_0000 + 1,
      prescaler : 32'h0100_0000 + 2,
      loadfactor: 32'h0100_0000 + 3,
      seed      : 32'h0100_0000 + 4,
      period    : 32'h0100_0000 + 5
  };

  typedef enum int unsigned {
    TASK_MBX = 0,
    TASK_UPD_0 = 1,
    TASK_UPD_1 = 2,
    TASK_UPD_2 = 3,
    TASK_UPD_3 = 4,
    TASK_CTRL_0 = 5,
    TASK_CTRL_1 = 6,
    TASK_CTRL_2 = 7,
    TASK_CTRL_3 = 8,
    TASK_REP_0 = 9,
    TASK_REP_1 = 10,
    TASK_REP_2 = 11,
    TASK_REP_3 = 12
  } task_num_e;

  localparam int unsigned TaskSetSize = 13;

  typedef struct packed {
    bit          active;
    logic [31:0] dl_us;
    task_num_e   name;
  } task_t;

  typedef struct packed {
    int unsigned count;
    int unsigned slack_worst;
    int unsigned slack_avg;
  } task_ret_t;

  function automatic int unsigned infer_mbx_per(input int unsigned dl, input int unsigned lf);
    // Use as reference in .rs source
    return 5 * dl + (4 * (100 - lf));
  endfunction

  function automatic task_addr_t get_task_addrs(input int unsigned task_idx);

    automatic logic [31:0] TaskBase = 32'h0200_0000 + (task_idx * 32'h1_0000);
    localparam logic [31:0] PerOffs = 32'h0;
    localparam logic [31:0] DlOffs = 32'h1;
    localparam logic [31:0] AckOffs = 32'h2;

    return task_addr_t
'{period : TaskBase + PerOffs, deadline : TaskBase + DlOffs, ack : TaskBase + AckOffs}
    ;
  endfunction

  localparam logic [31:0] PrintAddr = 32'h0300_0000;

endpackage
