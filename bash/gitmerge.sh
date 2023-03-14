#!/bin/bash

# Get the name of the current branch
current_branch=$(git rev-parse --abbrev-ref HEAD)

# Ask for confirmation to merge the current branch
read -p "🤔	Are you sure you want to merge branch $current_branch? [y/n] " -n 1 -r
echo    # move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]
then
		# Change to the main branch
		git checkout main
		
		# Merge the previously accepted branch
		git merge $current_branch

		if [ ! $? -eq 0 ]
		then
			echo "❌	Merge failed"
			exit 1
		fi
		
		# Push the result
		git push origin main
		
		# Check if the push was successful
		if [ $? -eq 0 ]
		then
				# Delete the branch at origin
				git push origin --delete $current_branch
				
				# Delete the local branch
				git branch -d $current_branch

				echo "🎉	Branch $current_branch merged successfully"
		else
				echo "❌	Push failed, branch not deleted"
		fi
else
	echo "🚫	Git merge aborted."
fi
