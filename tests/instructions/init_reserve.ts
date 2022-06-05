import { BN } from "@project-serum/anchor";
import { getPayer, program } from "../common";
import { TestLendingMarket } from "../helpers/test_lending_market";

describe("init_reserve", () => {
  let lendingMarket: TestLendingMarket;

  beforeEach(async () => {
    lendingMarket = await (await TestLendingMarket.init()).createLendingMarket();
  });

  describe("proper initialization", () => {});
});
