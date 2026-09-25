import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { z } from "zod";

import { request, requestImage, signInLocation } from "./client";
import { ConflictError, ThrottledError, ValidationError } from "./errors";

const schema = z.object({ name: z.string() });

function respond(status: number, body: string, headers: Record<string, string> = {}) {
  vi.stubGlobal("fetch", vi.fn(async () => new Response(body, { status, headers })));
}

describe("request", () => {
  const assign = vi.fn();

  beforeEach(() => {
    vi.stubGlobal("location", { pathname: "/services/", search: "?id=nas", assign });
    Object.defineProperty(window, "location", { value: { pathname: "/services/", search: "?id=nas", assign }, writable: true });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    assign.mockReset();
  });

  it("returns parsed data with the revision", async () => {
    respond(200, JSON.stringify({ name: "admin" }), { ETag: '"abc"' });
    await expect(request("/api/session", { schema })).resolves.toEqual({ data: { name: "admin" }, revision: '"abc"' });
  });

  it("sends the revision it loaded as If-Match", async () => {
    respond(200, JSON.stringify({ name: "x" }));
    await request("/api/x", { method: "PUT", body: {}, revision: '"r1"', schema });
    const [, init] = vi.mocked(fetch).mock.calls[0];
    expect((init?.headers as Record<string, string>)["If-Match"]).toBe('"r1"');
    expect((init?.headers as Record<string, string>)["Content-Type"]).toBe("application/json");
  });

  it("goes to the sign-in page remembering where it was on 401", async () => {
    respond(401, "sign in required");
    await expect(request("/api/services", { schema })).rejects.toThrow();
    expect(assign).toHaveBeenCalledWith("/login/?next=%2Fservices%2F%3Fid%3Dnas");
  });

  it("does not redirect when asked not to", async () => {
    respond(401, "sign in required");
    await expect(request("/api/session", { schema, redirectOnUnauthorized: false })).rejects.toThrow();
    expect(assign).not.toHaveBeenCalled();
  });

  it("maps 422 to field errors", async () => {
    respond(422, JSON.stringify({ errors: [{ field: "url", message: "bad" }] }));
    const error = await request("/api/services", { schema }).catch((caught: unknown) => caught);
    expect(error).toBeInstanceOf(ValidationError);
    expect((error as ValidationError).fields).toEqual([{ field: "url", message: "bad" }]);
  });

  it("maps 409 to a conflict and 429 to throttling with its delay", async () => {
    respond(409, "stale");
    await expect(request("/api/x", { schema })).rejects.toBeInstanceOf(ConflictError);
    respond(429, "slow down", { "Retry-After": "42" });
    const error = await request("/api/x", { schema }).catch((caught: unknown) => caught);
    expect((error as ThrottledError).retryAfterSeconds).toBe(42);
  });
});

describe("signInLocation", () => {
  it("keeps the path and query", () => {
    expect(signInLocation({ pathname: "/settings/network/", search: "" })).toBe("/login/?next=%2Fsettings%2Fnetwork%2F");
  });
});

describe("requestImage", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("posts the body as JSON and answers the image as a blob", async () => {
    const fetch = vi.fn(async () => new Response(new Uint8Array([137, 80]), { status: 200, headers: { "Content-Type": "image/png" } }));
    vi.stubGlobal("fetch", fetch);
    const blob = await requestImage("/api/icon-preview", { icon: "auto" });
    expect(blob.type).toBe("image/png");
    const [, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
    expect(init.method).toBe("POST");
    expect(init.body).toBe(JSON.stringify({ icon: "auto" }));
  });

  it("turns 422 into field errors", async () => {
    respond(422, JSON.stringify({ errors: [{ field: "icon", message: "not an image" }] }));
    await expect(requestImage("/api/icon-preview", {})).rejects.toBeInstanceOf(ValidationError);
  });
});
