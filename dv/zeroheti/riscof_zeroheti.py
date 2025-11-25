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
    __version__ = "0.1.0"

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        config = kwargs.get('config')
        if config is None:
            print("Please enter input file paths in configuration.")
            raise SystemExit(1)

        self.dut_exe = os.getcwd() + "/../build/verilator_build/obj_dir/Vzeroheti_compliance"
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

    def build(self, isa_yaml, platform_yaml):

      ispec = utils.load_yaml(isa_yaml)['hart0']
      self.xlen = ('64' if 64 in ispec['supported_xlen'] else '32')

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

      self.compile_cmd = self.compile_cmd+' -mabi='+('lp64 ' if 64 in ispec['supported_xlen'] else 'ilp32 ')

    def runTests(self, testList):
      if os.path.exists(self.work_dir+ "/Makefile." + self.name[:-1]):
            os.remove(self.work_dir+ "/Makefile." + self.name[:-1])
      # create an instance the makeUtil class that we will use to create targets.
      make = utils.makeUtil(makefilePath=os.path.join(self.work_dir, "Makefile." + self.name[:-1]))
      make.makeCommand = 'make -k -j' + self.num_jobs

      for testname in testList:
          testentry = testList[testname]
          test = testentry['test_path']
          test_dir = testentry['work_dir']
          td_parts = test_dir.split("/")
          root_dir = '/'.join(td_parts[:-5])

          name = "riscv"
          elf = f'{name}.elf'

          sig_file = os.path.join(test_dir, self.name[:-1] + ".signature")
          
          compile_macros= ' -D' + " -D".join(testentry['macros'])
          cc_cmd = self.compile_cmd.format(testentry['isa'].lower()+"_zicsr", self.xlen, test, elf, compile_macros)
         
          if self.target_run:
            simcmd = self.dut_exe + ' --isa={0} +signature={1} {2}'.format(self.isa, sig_file, elf)
          else:
            simcmd = 'echo "NO RUN"'

          execute = '@cd {0}; {1}; {2};'.format(testentry['work_dir'], cc_cmd, simcmd)
          make.add_target(execute)

      # if you would like to exit the framework once the makefile generation is complete uncomment the
      # following line. Note this will prevent any signature checking or report generation.
      raise SystemExit

      make.execute_all(self.work_dir)

      # if target runs are not required then we simply exit as this point after running all
      # the makefile targets.
      if not self.target_run:
          raise SystemExit(0)

