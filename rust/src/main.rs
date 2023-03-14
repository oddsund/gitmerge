use git2::{BranchType, MergeOptions, Repository};
use std::io::{self, Write};
use git2::FileMode::Tree;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open the current repository
    let repo = Repository::open(".")?;

    // Get the current branch name
    let head_ref = repo.head()?;
    let branch_head = repo.find_annotated_commit(head_ref.target().unwrap())?;
    let branch_name = head_ref.shorthand().unwrap();

    // Ask the user for confirmation if the current branch is to be merged
    print!("Do you want to merge {}? (y/n) ", branch_name);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim() != "y" {
        println!("Aborting merge.👋");
        return Ok(());
    }

    // Change to the main branch
    let main_branch = repo.find_branch("main", BranchType::Local)?;
    repo.set_head(main_branch.get().name().unwrap())?;

    // Check the result of the merge
    let (merge_analysis, _merge_preferences) = repo.merge_analysis(&[&branch_head])?;

    if !merge_analysis.is_fast_forward() && !merge_analysis.is_normal() {
        println!("Merge failed.😢");
        return Err("Merge failed".into());
    }

    let branch_commit = repo.find_commit(branch_head.target().unwrap())?;

    if merge_analysis.is_fast_forward() {
        println!("Fast-forward merge.🚀");

        repo.branch("refs/heads/main", &branch_commit, true)?;
    } else if merge_analysis.is_normal() {
        println!("Normal merge.👍");

        // Normal merge: need to create a commit object and update HEAD
        let signature = repo.signature()?;
        let main_commit = repo.find_commit(main_branch.get().target().unwrap())?;

        repo.merge(&[&branch_head], None, None)?;

        if repo.index().unwrap().has_conflicts() {
            println!("Merge failed with merge conflicts.😢");
            return Err("Merge failed".into());
        }

        // Create a commit object with two parents
        let _commit_id = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("Merge {} into main", branch_name),
            &local_tree,
            &[&branch_commit, &main_commit],
        )?;

        repo.cleanup_state()?;
    }
    // Push the result
    push_to_origin(&repo)?;
    // Delete remote and local branches
    delete_remote_branch(&repo, &branch_name)?;
    delete_local_branch(&repo, &branch_name)?;

    Ok(())
}

// Helper function to push changes to origin
fn push_to_origin(repo: &Repository) -> Result<(), Box<dyn std::error::Error>> {
    println!("Pushing changes to origin...");
    let mut remote = repo.find_remote("origin")?;
    remote.push(
        &[String::from("refs/heads/main")],
        Some(git2::PushOptions::new()),
    )?;
    println!("Push successful!✅");
    Ok(())
}

// Helper function to delete remote branch
fn delete_remote_branch(
    repo: &Repository,
    branch_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Deleting remote branch {}...", branch_name);
    let mut remote = repo.find_remote("origin")?;
    remote.push(
        &[format!(":{}", branch_name)],
        Some(git2::PushOptions::new()),
    )?;
    println!("Remote branch deleted!🗑️");
    Ok(())
}

// Helper function to delete local branch
fn delete_local_branch(
    repo: &Repository,
    branch_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Deleting local branch {}...", branch_name);
    let mut branch = repo.find_branch(branch_name, BranchType::Local)?;
    branch.delete()?;
    println!("Local branch deleted!🗑️");
    Ok(())
}
