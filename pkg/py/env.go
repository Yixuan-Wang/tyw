package py

import (
	"bytes"
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"path/filepath"
)

func ListEnv() {
	envHome := pyConfig.GetString("env.home")

	// Check if the path exists
	if envStat, err := os.Stat(envHome); os.IsNotExist(err) || !envStat.IsDir() {
		slog.Error("Path does not exist or is not a directory", "path", envHome)
		os.Exit(1)
	}

	// A Python venv is a directory with the file pyvenv.cfg
	// Walk through all subdirectories and check if they are Python venvs
	filepath.Walk(envHome, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			slog.Error("Cannot walk through path", "path", path, "error", err)
			os.Exit(1)
		}
		if info.IsDir() {
			if _, err := os.Stat(path + "/pyvenv.cfg"); err == nil {
				relpath, _ := filepath.Rel(envHome, path)
				fmt.Println(relpath)
			}
		}
		return nil
	})
}

// Given an environment name, print the command to activate the environment
func UseEnv(name string) {
	envHome := pyConfig.GetString("env.home")

	if name == "" {
		slog.Error("Environment name is empty")
		os.Exit(1)
	}

	env := filepath.Join(envHome, name)

	// Check if the path exists
	if envStat, err := os.Stat(env); os.IsNotExist(err) || !envStat.IsDir() {
		slog.Error("Path does not exist or is not a directory", "path", env)
		os.Exit(1)
	}

	// Check if the environment exists
	if _, err := os.Stat(envHome + "/" + name + "/pyvenv.cfg"); os.IsNotExist(err) {
		slog.Error("Environment does not exist", "name", name)	
		os.Exit(1)
	}

	// Print the command to activate the environment
	fmt.Printf("%s", fmt.Sprintf("source %s/bin/activate", env))
}

// List all Python virtual environments, pipe to `fzf` for selection
// and then print the line to activate the selected environment 
func UseEnvQ() {
	envHome := pyConfig.GetString("env.home")

	// Check if the path exists
	if envStat, err := os.Stat(envHome); os.IsNotExist(err) || !envStat.IsDir() {
		slog.Error("Environment home does not exist or is not a directory", "path", envHome)
	}

	// A Python venv is a directory with the file pyvenv.cfg
	// Walk through all subdirectories and check if they are Python venvs
	envs := make(map[string]string)
	filepath.Walk(envHome, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			slog.Error("Cannot walk through path", "path", path, "error", err)
			os.Exit(1)
		}
		if info.IsDir() {
			if _, err := os.Stat(path + "/pyvenv.cfg"); err == nil {
				relpath, _ := filepath.Rel(envHome, path)
				envs[relpath] = path
			}
		}
		return nil
	})

	var input bytes.Buffer
	for k := range envs {
		input.WriteString(k + "\n")
	}

	// Pipe the list of environments to fzf
	// fzf will return the selected environment
	fzf := exec.Command("fzf")
	fzf.Stdin = &input
	output, err := fzf.Output()
	if err != nil {
		slog.Error("Failed to run fzf", "error", err)
		os.Exit(1)
	}

	// Print the command to activate the selected environment without an intermediate variable
	fmt.Printf("source %s/bin/activate\n", envs[string(bytes.TrimSpace(output))])
}

func TryUseEnv() {
	// Get current working directory
	cwd, err := os.Getwd()
	if err != nil {
		slog.Error("Failed to get current working directory", "error", err)
		os.Exit(1)
	}

	// Check if the path exists
	if envStat, err := os.Stat(cwd); os.IsNotExist(err) || !envStat.IsDir() {
		slog.Error("Path does not exist or is not a directory", "path", cwd)
		os.Exit(1)
	}

	// TODO: Should find the nearest venv of arbitrary name
	// Now just walk up the directory tree and check if there is a venv
	//   of name `venv` or `.venv`

	dir := cwd
	for {
		if _, err := os.Stat(dir + "/venv/pyvenv.cfg"); err == nil {
			fmt.Printf("source %s/venv/bin/activate\n", dir)
			break
		}
		if _, err := os.Stat(dir + "/.venv/pyvenv.cfg"); err == nil {
			fmt.Printf("source %s/.venv/bin/activate\n", dir)
			break
		}
		if filepath.Dir(dir) == dir {
			slog.Error("No venv found")
			os.Exit(1)
		}
		dir = filepath.Dir(dir)
	}
}