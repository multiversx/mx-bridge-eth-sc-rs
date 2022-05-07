read -e -p "Deploy?:" -i "Yes" CONTINUE
echo    # (optional) move to a new line
if [[ "$CONTINUE" = "Yes" ]]
then
    echo "exiting..."
    [[ "$CONTINUE" = "Yes" ]] && exit 1 || return 1 # handle exits from shell or function but don't exit interactive shell
fi

echo "Success