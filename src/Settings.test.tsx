// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import Settings from "./Settings";
import { FORK_PRESETS } from "./types/fork";

describe("Settings", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders preset fork buttons", () => {
    render(
      <Settings selectedFork={FORK_PRESETS.official} onSelectFork={vi.fn()} />,
    );

    expect(screen.getByText("MinUI (Official)")).toBeInTheDocument();
    expect(screen.getByText("MinUI-Zero")).toBeInTheDocument();
    expect(screen.getByText("shauninman/MinUI")).toBeInTheDocument();
    expect(screen.getByText("danklammer/MinUI-Zero")).toBeInTheDocument();
  });

  it("highlights the active preset", () => {
    render(
      <Settings
        selectedFork={FORK_PRESETS["minui-zero"]}
        onSelectFork={vi.fn()}
      />,
    );

    const zeroBtn = screen.getByText("MinUI-Zero").closest("button");
    expect(zeroBtn?.className).toContain("active");

    const officialBtn = screen.getByText("MinUI (Official)").closest("button");
    expect(officialBtn?.className).not.toContain("active");
  });

  it("calls onSelectFork when a preset is clicked", async () => {
    const onSelectFork = vi.fn();
    render(
      <Settings
        selectedFork={FORK_PRESETS.official}
        onSelectFork={onSelectFork}
      />,
    );

    await userEvent.click(screen.getByText("MinUI-Zero"));
    expect(onSelectFork).toHaveBeenCalledWith(FORK_PRESETS["minui-zero"]);
  });

  it("shows error for invalid custom input", async () => {
    render(
      <Settings selectedFork={FORK_PRESETS.official} onSelectFork={vi.fn()} />,
    );

    const input = screen.getByPlaceholderText("owner/repo");
    await userEvent.type(input, "not valid");
    await userEvent.click(screen.getByText("Use"));

    expect(screen.getByText(/Invalid format/)).toBeInTheDocument();
  });

  it("calls onSelectFork with valid custom fork", async () => {
    const onSelectFork = vi.fn();
    render(
      <Settings
        selectedFork={FORK_PRESETS.official}
        onSelectFork={onSelectFork}
      />,
    );

    const input = screen.getByPlaceholderText("owner/repo");
    await userEvent.type(input, "my-user/my-repo");
    await userEvent.click(screen.getByText("Use"));

    expect(onSelectFork).toHaveBeenCalledWith({
      label: "my-user/my-repo",
      owner: "my-user",
      repo: "my-repo",
      versionPrefix: "my-repo",
    });
  });

  it("submits custom fork on Enter key", async () => {
    const onSelectFork = vi.fn();
    render(
      <Settings
        selectedFork={FORK_PRESETS.official}
        onSelectFork={onSelectFork}
      />,
    );

    const input = screen.getByPlaceholderText("owner/repo");
    await userEvent.type(input, "org/fork{Enter}");

    expect(onSelectFork).toHaveBeenCalledTimes(1);
  });

  it("shows active fork label for custom forks not in presets", () => {
    const customFork = {
      label: "custom/thing",
      owner: "custom",
      repo: "thing",
      versionPrefix: "thing",
    };
    render(<Settings selectedFork={customFork} onSelectFork={vi.fn()} />);

    expect(screen.getByText(/Active:/)).toBeInTheDocument();
    expect(screen.getByText("custom/thing")).toBeInTheDocument();
  });

  it("does not show active fork label for preset forks", () => {
    render(
      <Settings selectedFork={FORK_PRESETS.official} onSelectFork={vi.fn()} />,
    );

    expect(screen.queryByText(/Active:/)).not.toBeInTheDocument();
  });
});
