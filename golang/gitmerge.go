package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/transport"
)

func main() {
	// Open the Git repository in the current directory
	r, err := git.PlainOpen(".")
	checkIfError(err)

	// Get the name of the current branch
	headRef, err := r.Head()
	checkIfError(err)
	currentBranch := strings.TrimPrefix(headRef.Name().String(), "refs/heads/")

	// Ask for confirmation to merge the current branch
	reader := bufio.NewReader(os.Stdin)
	fmt.Printf("🤔 Are you sure you want to merge branch %s? [y/n] ", currentBranch)
	confirm, _ := reader.ReadString('\n')
	if strings.TrimSpace(confirm) != "y" {
		fmt.Println("🚫 Merge cancelled")
		return
	}

	// Change to the main branch
	mainBranch := "main"
	ref := plumbing.NewBranchReferenceName(mainBranch)
	mainRef, err := r.Reference(ref, true)
	checkIfError(err)
	w, err := r.Worktree()
	checkIfError(err)
	err = w.Checkout(&git.CheckoutOptions{
		Hash: mainRef.Hash(),
	})
	checkIfError(err)

	// Merge the previously accepted branch
	mergeRef := plumbing.NewBranchReferenceName(currentBranch)
	mergeRefObj, err := r.Reference(mergeRef, true)
	checkIfError(err)
	mergeCommit, err := r.CommitObject(mergeRefObj.Hash())
	checkIfError(err)
	mergeOptions := &git.MergeOptions{
		Commit: mergeCommit,
	}
	mergeAnalysis, _, err := r.Merge(mergeRefObj.Hash(), mergeOptions)
	checkIfError(err)
	if mergeAnalysis == git.MergeAnalysisAlreadyUpToDate {
		fmt.Println("👍 Branch is already up-to-date")
		return
	} else if mergeAnalysis != git.MergeAnalysisFastForward {
		fmt.Println("❌ Merge is not a fast-forward")
		return
	}

	// Push the result
	pushOptions := &git.PushOptions{
		RemoteName: "origin",
		Auth:       getAuthMethod(),
	}
	err = r.Push(pushOptions)
	if err != nil {
		fmt.Println("❌ Push failed")
		return
	}

	// Delete the branch at origin only if the push was successful
	err = r.DeleteRemoteBranch("origin", currentBranch)
	if err != nil {
		fmt.Println("❌ Deletion of branch at origin failed")
		return
	}

	// Delete the local branch
	err = r.DeleteBranch(currentBranch)
	checkIfError(err)

	fmt.Println("✅ Merge successful")
}

func getAuthMethod() transport.AuthMethod {
	// Insert code here to get the authentication credentials.
	// See https://pkg.go.dev/github.com/go-git/go-git/v5/plumbing/transport#AuthMethod for options.
	return nil
}

func checkIfError(err error) {
	if err != nil {
		fmt.Println("❌", err)
		os.Exit(1)
	}
}

