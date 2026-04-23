#ifndef MOTOR_H
#define MOTOR_H

#define NUM_MOTORS  4

/* I2C slave address map */
#define I2C_SIM_CTRL_ADDR  0
#define I2C_M0_STAT_ADDR   1
#define I2C_M0_CTRL_ADDR   2
#define I2C_M0_TUNE_ADDR   3
#define I2C_M1_STAT_ADDR   4
#define I2C_M1_CTRL_ADDR   5
#define I2C_M1_TUNE_ADDR   6
#define I2C_M2_STAT_ADDR   7
#define I2C_M2_CTRL_ADDR   8
#define I2C_M2_TUNE_ADDR   9
#define I2C_M3_STAT_ADDR   10
#define I2C_M3_CTRL_ADDR   11
#define I2C_M3_TUNE_ADDR   12

static const uint8_t stat_m[NUM_MOTORS] = {
    I2C_M0_STAT_ADDR, I2C_M1_STAT_ADDR,
    I2C_M2_STAT_ADDR, I2C_M3_STAT_ADDR
};
static const uint8_t ctrl_m[NUM_MOTORS] = {
    I2C_M0_CTRL_ADDR, I2C_M1_CTRL_ADDR,
    I2C_M2_CTRL_ADDR, I2C_M3_CTRL_ADDR
};
static const uint8_t tune_m[NUM_MOTORS] = {
    I2C_M0_TUNE_ADDR, I2C_M1_TUNE_ADDR,
    I2C_M2_TUNE_ADDR, I2C_M3_TUNE_ADDR
};

#endif