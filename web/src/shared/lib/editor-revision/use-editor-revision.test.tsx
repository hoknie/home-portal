import { act, renderHook } from "@testing-library/react";
import { expect, it } from "vitest";

import { useEditorRevision } from "./use-editor-revision";

it("keeps the revision the values were loaded with while the list refreshes", () => {
  const { result, rerender } = renderHook(({ latest }) => useEditorRevision(latest), { initialProps: { latest: '"r1"' as string | null } });
  rerender({ latest: '"r2"' });
  expect(result.current.revision).toBe('"r1"');
  expect(result.current.latest).toBe('"r2"');
});

it("takes the first revision that arrives after loading", () => {
  const { result, rerender } = renderHook(({ latest }) => useEditorRevision(latest), { initialProps: { latest: null as string | null } });
  rerender({ latest: '"r1"' });
  rerender({ latest: '"r2"' });
  expect(result.current.revision).toBe('"r1"');
});

it("moves on only after its own save or when the person catches up", () => {
  const { result, rerender } = renderHook(({ latest }) => useEditorRevision(latest), { initialProps: { latest: '"r1"' as string | null } });
  act(() => result.current.adopt('"r5"'));
  expect(result.current.revision).toBe('"r5"');
  rerender({ latest: '"r6"' });
  let caught: string | null = null;
  act(() => {
    caught = result.current.catchUp();
  });
  expect(caught).toBe('"r6"');
  expect(result.current.revision).toBe('"r6"');
});
