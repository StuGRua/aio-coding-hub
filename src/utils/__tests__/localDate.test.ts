import { describe, expect, it } from "vitest";
import {
  parseYyyyMmDd,
  unixSecondsAtLocalStartOfDay,
  unixSecondsAtLocalStartOfNextDay,
} from "../localDate";

describe("utils/localDate", () => {
  it("parseYyyyMmDd validates input", () => {
    expect(parseYyyyMmDd("")).toBeNull();
    expect(parseYyyyMmDd("2020/01/01")).toBeNull();
    expect(parseYyyyMmDd("2020-00-01")).toBeNull();
    expect(parseYyyyMmDd("2020-13-01")).toBeNull();
    expect(parseYyyyMmDd("2020-01-00")).toBeNull();
    expect(parseYyyyMmDd("2020-01-32")).toBeNull();
    expect(parseYyyyMmDd("2020-01-01")).toEqual({ year: 2020, month: 1, day: 1 });
  });

  it("unix seconds helpers return numbers", () => {
    const start = unixSecondsAtLocalStartOfDay("2020-01-01");
    const next = unixSecondsAtLocalStartOfNextDay("2020-01-01");
    expect(typeof start).toBe("number");
    expect(typeof next).toBe("number");
    expect(next).toBeGreaterThan(start as number);
  });

  it("returns null for invalid date strings in unix helpers", () => {
    expect(unixSecondsAtLocalStartOfDay("invalid")).toBeNull();
    expect(unixSecondsAtLocalStartOfNextDay("2020-99-99")).toBeNull();
  });

  it("keeps next-day delta within one day around DST transitions", () => {
    const start = unixSecondsAtLocalStartOfDay("2024-03-10");
    const next = unixSecondsAtLocalStartOfNextDay("2024-03-10");
    expect(start).not.toBeNull();
    expect(next).not.toBeNull();

    const delta = (next as number) - (start as number);
    expect(delta).toBeGreaterThanOrEqual(23 * 3600);
    expect(delta).toBeLessThanOrEqual(25 * 3600);
  });
});
