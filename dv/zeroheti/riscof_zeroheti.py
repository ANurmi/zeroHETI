import os
import re
import shutil
import subprocess
import shlex
import logging
import random
import string
from string import Template
import sys

import riscof.utils as utils
import riscof.constants as constants
from riscof.pluginTemplate import pluginTemplate

logger = logging.getLogger()

class zeroheti(pluginTemplate):
    __model__ = "zeroheti"

    #TODO: please update the below to indicate family, version, etc of your DUT.
    __version__ = "0.1.0"

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        config = kwargs.get('config')
        if config is None:
            print("Please enter input file paths in configuration.")
            raise SystemExit(1)

        self.dut_exe = os.getcwd() + "/../build/verilator_build/obj_dir/Vzeroheti_top"
        self.num_jobs = str(config['jobs'] if 'jobs' in config else 1)
        self.pluginpath=os.path.abspath(config['pluginpath'])
        self.isa_spec = os.path.abspath(config['ispec'])
        self.platform_spec = os.path.abspath(config['pspec'])
        if 'target_run' in config and config['target_run']=='0':
            self.target_run = False
        else:
            self.target_run = True

    def initialise(self, suite, work_dir, archtest_env):
       self.work_dir = work_dir
       self.suite_dir = suite

       self.compile_cmd = 'riscv{1}-unknown-elf-gcc -march={0} \
         -static -mcmodel=medany -fvisibility=hidden -nostdlib -nostartfiles -g\
         -T '+self.pluginpath+'/env/link.ld\
         -I '+self.pluginpath+'/env/\
         -I ' + archtest_env + ' {2} -o {3} {4}'

       # add more utility snippets here

    def build(self, isa_yaml, platform_yaml):

      # load the isa yaml as a dictionary in python.
      ispec = utils.load_yaml(isa_yaml)['hart0']

      # capture the XLEN value by picking the max value in 'supported_xlen' field of isa yaml. This
      # will be useful in setting integer value in the compiler string (if not already hardcoded);
      self.xlen = ('64' if 64 in ispec['supported_xlen'] else '32')

      # for zeroheti start building the '--isa' argument. the self.isa is dutnmae specific and may not be
      # useful for all DUTs
      self.isa = 'rv' + self.xlen
      if "I" in ispec["ISA"]:
          self.isa += 'i'
      if "M" in ispec["ISA"]:
          self.isa += 'm'
      if "F" in ispec["ISA"]:
          self.isa += 'f'
      if "D" in ispec["ISA"]:
          self.isa += 'd'
      if "C" in ispec["ISA"]:
          self.isa += 'c'

      #TODO: The following assumes you are using the riscv-gcc toolchain. If
      #      not please change appropriately
      self.compile_cmd = self.compile_cmd+' -mabi='+('lp64 ' if 64 in ispec['supported_xlen'] else 'ilp32 ')

    def runTests(self, testList):
      # Delete Makefile if it already exists.
      if os.path.exists(self.work_dir+ "/Makefile." + self.name[:-1]):
            os.remove(self.work_dir+ "/Makefile." + self.name[:-1])
      # create an instance the makeUtil class that we will use to create targets.
      make = utils.makeUtil(makefilePath=os.path.join(self.work_dir, "Makefile." + self.name[:-1]))
      make.makeCommand = 'make -k -j' + self.num_jobs

      # we will iterate over each entry in the testList. Each entry node will be refered to by the
      # variable testname.
      for testname in testList:
          testentry = testList[testname]
          test = testentry['test_path']
          test_dir = testentry['work_dir']
          td_parts = test_dir.split("/")
          root_dir = '/'.join(td_parts[:-5])
          build_dir = root_dir + "/build"
          vbuild_dir = build_dir + "/verilator_build"
          vstim = vbuild_dir + "/verilator_stim.hex"
          istim = vbuild_dir + "/imem_stim.hex"
          dstim = vbuild_dir + "/dmem_stim.hex"

          imem_size  = 5120
          dmem_start = 16385

          name = "riscv"
          elf = f'{name}.elf'
          testbin = f'{name}.bin'
          testhex = f'{name}.hex'

          sig_file = os.path.join(test_dir, self.name[:-1] + ".signature")

          # for each test there are specific compile macros that need to be enabled. The macros in
          # the testList node only contain the macros/values. For the gcc toolchain we need to
          # prefix with "-D".
          compile_macros= ' -D' + " -D".join(testentry['macros'])

          # substitute all variables in the compile command that we created in the initialize
          # function
          cc_cmd = self.compile_cmd.format(testentry['isa'].lower(), self.xlen, test, elf, compile_macros)
         
          
          # reuse formating from examples/smoke_tests
          hex_cmd = f'mkdir -p {build_dir}/sw && cp {elf} {build_dir}/sw && cd {root_dir} && make -C examples/smoke_tests hex trim TEST={name}'
          fmt_cmd = f'cp {build_dir}/sw/{testhex} {vstim} && head -{imem_size} {vstim} > {istim} && tail -n +{dmem_start} {vstim} > {dstim}'

	  # if the user wants to disable running the tests and only compile the tests, then
	  # the "else" clause is executed below assigning the sim command to simple no action
	  # echo statement.
          if self.target_run:
            simcmd = self.dut_exe + ' --isa={0} +signature={1} {2}'.format(self.isa, sig_file, elf)
          else:
            simcmd = 'echo "NO RUN"'

          # concatenate all commands that need to be executed within a make-target.
          execute = '@cd {0}; {1}; {2}; {3}; {4};'.format(testentry['work_dir'], cc_cmd, hex_cmd, fmt_cmd, simcmd)

          # create a target. The makeutil will create a target with the name "TARGET<num>" where num
          # starts from 0 and increments automatically for each new target that is added
          make.add_target(execute)

      # if you would like to exit the framework once the makefile generation is complete uncomment the
      # following line. Note this will prevent any signature checking or report generation.
      #raise SystemExit

      # once the make-targets are done and the makefile has been created, run all the targets in
      # parallel using the make command set above.
      make.execute_all(self.work_dir)

      # if target runs are not required then we simply exit as this point after running all
      # the makefile targets.
      if not self.target_run:
          raise SystemExit(0)

    def makehex(self, binfile, hexfile):
        pass
        #with open(binfile, "rb") as f, open(hexfile, "w") as fout:



