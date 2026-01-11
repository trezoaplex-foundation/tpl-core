import { UmiPlugin } from '@trezoaplex-foundation/umi';
import { createMplCoreProgram } from './generated';

export const tplCore = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createMplCoreProgram(), false);
  },
});
