package util

import (
	"bytes"
	"fmt"
	"io"
	"log/slog"
	"os"
	"os/exec"
	"strings"
)

type FzfLine[V any] struct {
	Key    string
	Pretty []string
	Raw    V
}

func FzfGetFromChan[K comparable, V any](
	choices <-chan K,
	fn func(K) (FzfLine[V], error),
	arg ...string,
) (V, error) {
	arg = append([]string{"--with-nth", "2.."}, arg...)

	fzf := exec.Command("fzf", arg...)
	mapping := make(map[string]V)
	out := make(chan V, 1)
	var zero V

	pipeIn, err := fzf.StdinPipe()
	if err != nil {
		slog.Error("Failed to create stdin pipe of fzf", "error", err)
		return zero, err
	}

	pipeOut, err := fzf.StdoutPipe()
	if err != nil {
		slog.Error("Failed to create stdout pipe of fzf", "error", err)
		return zero, err
	}

	// fzf's stderr to this process
	fzf.Stderr = os.Stderr

	go func() {
		defer pipeIn.Close()
		for choice := range choices {

			line, err := fn(choice)
			if err != nil {
				continue
			}
			mapping[line.Key] = line.Raw
			if _, err := fmt.Fprintf(pipeIn, "%s %s\n", line.Key, strings.Join(line.Pretty, " ")); err != nil {
				slog.Error("Failed to write to fzf stdin", "error", err)
				return
			}
		}

	}()

	go func() {
		output, err := io.ReadAll(pipeOut)
		if err != nil {
			slog.Error("Failed to run fzf", "error", err)
			return
		}

		outBytesFirst, _, _ := bytes.Cut(output, []byte(" "))
		out <- mapping[string(outBytesFirst)]
	}()

	if err := fzf.Run(); err != nil {
		slog.Error("Failed to run fzf", "error", err)
		return zero, err
	}
	return <-out, nil
}
