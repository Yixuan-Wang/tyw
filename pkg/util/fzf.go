package util

import (
	"bytes"
	"fmt"
	"log/slog"
	"os/exec"
	"strings"
)

func FzfSelectFromChan[T any](
	choices <-chan T,
	fn func(T) (string, error),
	arg ...string,
) (<-chan string, error) {
	fzf := exec.Command("fzf", arg...)
	pipe, err := fzf.StdinPipe()
	if err != nil {
		slog.Error("Failed to create stdin pipe of fzf", "error", err)
		return nil, err
	}

	go func() {
		defer pipe.Close()
		for choice := range choices {
			inStr, err := fn(choice)
			if err != nil {
				continue
			}
			if _, err := fmt.Fprintln(pipe, inStr); err != nil {
				slog.Error("Failed to write to fzf stdin", "error", err)
				return
			}
		}
	}()

	out := make(chan string)

	go func() {
		defer close(out)
		output, err := fzf.Output()
		if err != nil {
			slog.Error("Failed to run fzf", "error", err)
			return
		}
		outStr := string(bytes.TrimSpace(output))
		out <- outStr
	}()

	return out, nil
}

type FzfLine[V any] struct {
	Key    string
	Pretty []string
	Raw    V
}

func FzfGetFromChan[K comparable, V any](
	choices <-chan K,
	fn func(K) (FzfLine[V], error),
	arg ...string,
) (<-chan V, error) {
	arg = append([]string{"--with-nth", "2.."}, arg...)

	fzf := exec.Command("fzf", arg...)
	pipe, err := fzf.StdinPipe()
	if err != nil {
		slog.Error("Failed to create stdin pipe of fzf", "error", err)
		return nil, err
	}

	mapping := make(map[string]V)
	go func() {
		defer pipe.Close()
		for choice := range choices {
			line, err := fn(choice)
			if err != nil {
				continue
			}
			mapping[line.Key] = line.Raw
			if _, err := fmt.Fprintf(pipe, "%s %s\n", line.Key, strings.Join(line.Pretty, " ")); err != nil {
				slog.Error("Failed to write to fzf stdin", "error", err)
				return
			}
		}
	}()

	out := make(chan V)

	go func() {
		defer close(out)
		output, err := fzf.Output()
		if err != nil {
			slog.Error("Failed to run fzf", "error", err)
			return
		}
		outBytesFirst, _, _ := bytes.Cut(output, []byte(" "))
		out <- mapping[string(outBytesFirst)]
	}()

	return out, nil
}
