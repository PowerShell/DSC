function Get-AvailablePackageManager
{
    if (Get-Command 'apt' -ErrorAction SilentlyContinue)
    {
        return 'apt'
    }

    if (Get-Command 'dnf' -ErrorAction SilentlyContinue)
    {
        return 'dnf'
    }

    if (Get-Command 'yum' -ErrorAction SilentlyContinue)
    {
        return 'yum'
    }

    if (Get-Command 'zypper' -ErrorAction SilentlyContinue)
    {
        return 'zypper'
    }
}