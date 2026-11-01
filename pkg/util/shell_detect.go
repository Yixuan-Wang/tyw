package util

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
)

// knownShells is a set of common shell names to validate the parent process.
var knownShells = map[string]bool{
	"sh":   true,
	"bash": true,
	"zsh":  true,
	"fish": true,
	"dash": true,
	"ksh":  true,
	"csh":  true,
	"tcsh": true,
	"nu":   true,
}

// Attempts to identify the shell of the parent process.
// 
// If the parent process is not a known shell or cannot be detected,
// it falls back to the `$SHELL` environment variable.
func DetectShell() (string, error) {
	// 1. Attempt to find the executable path of the parent process (PPID)
	ppid := os.Getppid()
	parentPath, err := getProcessPath(ppid)

	// 2. If successful, check if the parent is actually a shell
	if err == nil && parentPath != "" {
		name := filepath.Base(parentPath)
		// Handle cases where the binary might be named "zsh-5.8" or similar,
		// though usually shells are just "zsh". We stick to exact matches or
		// simple prefix checks if necessary. Here we do exact match on common names.
		if knownShells[name] {
			return name, nil
		}
	}

	// 3. Fallback: If parent detection failed or parent wasn't a shell, use $SHELL
	envShell := os.Getenv("SHELL")
	if envShell == "" {
		return "", errors.New("could not detect parent shell and $SHELL is unset")
	}

	return filepath.Base(envShell), nil
}

// getProcessPath returns the absolute path to the executable of a given PID.
func getProcessPath(pid int) (string, error) {
	if runtime.GOOS == "linux" {
		// On Linux, reading the symlink /proc/<pid>/exe is the most robust method
		link := fmt.Sprintf("/proc/%d/exe", pid)
		return os.Readlink(link)
	} else if runtime.GOOS == "darwin" {
		// On macOS, /proc doesn't exist. We use `ps`.
		// -p: pid, -o comm=: output only command (executable path)
		cmd := exec.Command("ps", "-p", fmt.Sprintf("%d", pid), "-o", "comm=")
		var out bytes.Buffer
		cmd.Stdout = &out
		if err := cmd.Run(); err != nil {
			return "", err
		}
		return strings.TrimSpace(out.String()), nil
	}
	
	// Fallback for other unix-likes if needed, or just fail
	return "", fmt.Errorf("unsupported platform for process lookup: %s", runtime.GOOS)
}
