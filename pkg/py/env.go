package py

import (
	"bufio"
	"container/list"
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"regexp"

	"github.com/yixuan-wang/tyw/pkg/util"
)

// Walk through the subtree of the given path and find all Python virtual environments,
// identified by the presence of a pyvenv.cfg file.
//
// The root directory should be guaranteed to exist and be a directory.
func walkDirForVenv(root string, out chan<- string) error {
	defer close(out)
	queue := list.New()
	queue.PushBack(root)

	for queue.Len() > 0 {
		elem := queue.Front()
		queue.Remove(elem)
		dir := elem.Value.(string)

		// Check if the directory contains a Python venv
		// Python env is marked by pyvenv.cfg file
		maybeVenvCfg := dir + "/pyvenv.cfg"
		if _, err := os.Stat(maybeVenvCfg); os.IsNotExist(err) {
			// Not a Python venv, push all subdirectories to the queue
			if files, err := os.ReadDir(dir); err == nil {
				for _, file := range files {
					if file.IsDir() {
						subdir := filepath.Join(dir, file.Name())
						queue.PushBack(subdir)
					}
				}
			} else {
				slog.Error("Cannot read directory", "path", dir, "error", err)
				return err
			}
		} else {
			out <- dir
		}
	}

	return nil
}

type VenvInfo struct {
	Home    string
	Version string
	Prompt  string
}

var regexHome = regexp.MustCompile(`^home\s*=\s*(.*)`)
var regexVersion = regexp.MustCompile(`^version(?:_info)?\s*=\s*(.*)`)
var regexPrompt = regexp.MustCompile(`^prompt\s*=\s*(.*)`)

func getVenvInfo(prefix string) (VenvInfo, error) {
	file, err := os.Open(filepath.Join(prefix, "pyvenv.cfg"))
	if err != nil {
		return VenvInfo{}, err
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	var info VenvInfo

	for scanner.Scan() {
		line := scanner.Text()
		if matches := regexHome.FindStringSubmatch(line); len(matches) > 1 {
			info.Home = matches[1]
		} else if matches := regexVersion.FindStringSubmatch(line); len(matches) > 1 {
			info.Version = matches[1]
		} else if matches := regexPrompt.FindStringSubmatch(line); len(matches) > 1 {
			info.Prompt = matches[1]
		}
	}

	if err := scanner.Err(); err != nil {
		return VenvInfo{}, err
	}

	return info, nil
}

func ListEnv() error {
	envHome := pyConfig.GetString("env.home")

	// Check if the path exists
	if envStat, err := os.Stat(envHome); os.IsNotExist(err) || !envStat.IsDir() {
		slog.Error("Path does not exist or is not a directory", "path", envHome)
		return nil
	}

	dirs := make(chan string)
	go func() {
		if err := walkDirForVenv(envHome, dirs); err != nil {
			slog.Error("Failed to walk directory", "path", envHome, "error", err)
			return
		}
	}()
	for dir := range dirs {
		// Print the directory name
		fmt.Println(dir)
	}

	return nil
}

// Given an environment name, print the command to activate the environment
func UseEnv(name string) error {
	envHome := pyConfig.GetString("env.home")

	if name == "" {
		slog.Error("Environment name is empty")
		return nil
	}

	env := filepath.Join(envHome, name)

	// Check if the path exists
	if envStat, err := os.Stat(env); os.IsNotExist(err) || !envStat.IsDir() {
		slog.Error("Path does not exist or is not a directory", "path", env)
		return nil
	}

	// Check if the environment exists
	if _, err := os.Stat(filepath.Join(envHome, name, "pyvenv.cfg")); os.IsNotExist(err) {
		slog.Error("Environment does not exist", "name", name)
		return nil
	}

	// Print the command to activate the environment
	fmt.Printf("%s", fmt.Sprintf("source %s", filepath.Join(env, "bin", "activate")))
	return nil
}

// List all Python virtual environments, pipe to `fzf` for selection
// and then print the line to activate the selected environment
func SelectEnv() error {
	envHome := pyConfig.GetString("env.home")

	// Check if the path exists
	if envStat, err := os.Stat(envHome); os.IsNotExist(err) || !envStat.IsDir() {
		slog.Error("Environment home does not exist or is not a directory", "path", envHome)
		return nil
	}

	// A Python venv is a directory with the file pyvenv.cfg
	// Walk through all subdirectories and check if they are Python venvs

	venvDirs := make(chan string)
	go func() {
		if err := walkDirForVenv(envHome, venvDirs); err != nil {
			slog.Error("Failed to walk directory", "path", envHome, "error", err)
			return
		}
	}()

	fzf, err := util.FzfGetFromChan(venvDirs, func(path string) (util.FzfLine[string], error) {
		relPath, _ := filepath.Rel(envHome, path)
		info, err := getVenvInfo(path)
		if err != nil {
			slog.Error("Failed to get venv info", "path", path, "error", err)
			return util.FzfLine[string]{}, err
		}

		var line util.FzfLine[string]
		line.Key = relPath
		line.Raw = path

		name := filepath.Base(relPath)

		if info.Prompt != "" && info.Prompt != name {
			line.Pretty = []string{fmt.Sprintf("%s(%s)", info.Prompt, relPath), info.Version}
		} else {
			line.Pretty = []string{name, info.Version}
		}
		return line, nil
	})
	if err != nil {
		slog.Error("Failed to initialize fzf", "error", err)
		return nil
	}

	// Print the command to activate the selected environment without an intermediate variable
	fmt.Printf("%s", fmt.Sprintf("source %s", filepath.Join(fzf, "bin", "activate")))
	return nil
}

func TryUseEnv() error {
	// Get current working directory
	cwd, err := os.Getwd()
	if err != nil {
		slog.Error("Failed to get current working directory", "error", err)
		return nil
	}

	// Check if the path exists
	if envStat, err := os.Stat(cwd); os.IsNotExist(err) || !envStat.IsDir() {
		return util.Fail("Path does not exist or is not a directory", "path", cwd)
	}

	dir := cwd
	for {
		if _, err := os.Stat(filepath.Join(dir, "venv", "pyvenv.cfg")); err == nil {
			fmt.Printf("source %s\n", filepath.Join(dir, "venv", "bin", "activate"))
			break
		}
		if _, err := os.Stat(filepath.Join(dir, ".venv", "pyvenv.cfg")); err == nil {
			fmt.Printf("source %s\n", filepath.Join(dir, ".venv", "bin", "activate"))
			break
		}
		if filepath.Dir(dir) == dir {
			return util.Fail("No virtual environment found in the directory tree")
		}
		dir = filepath.Dir(dir)
	}

	return nil
}
