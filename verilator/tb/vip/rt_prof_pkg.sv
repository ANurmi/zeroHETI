package rt_prof_pkg;

  typedef struct packed {
    logic [31:0] addr;
    logic [31:0] data;
  } letter_t;

  typedef struct packed {
    bit          available;
    bit          started;
    int          dl_cc;
    int          ret_worst_cc;
    int          ret_avg_cc;
    int unsigned dl_target_cc;
    int          count_total;
    int unsigned count_misses;
  } task_t;

endpackage
