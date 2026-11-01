package util

import (
	"context"
	"fmt"
	"log/slog"
)

func Fail(mst string, args ...any) error {
	slog.Log(context.Background(), 12, mst, args...)
	return fmt.Errorf("%s", mst)
}