# AI Instruction

1.
在当前文件夹下创建rust工程，编写 pmr 命令行工具，用于守护进程的管理：
1. 支持windowns、Linux、MacOS
2. 支持命令直接启动，例如  pmr start --name [别名] [程序名] -- [参数]
3. 支持配置文件启动，例如 pmr start --config config.json:
{
    "name": "test",
    "program": "python",
    "args": ["-m", "test"]
}
4. 支持查看进程列表，例如 pmr list（alias命令：ls），在终端显示列表，例如┌────┬───────────────┬─────────────┬─────────┬─────────┬──────────┬────────┬──────┬───────────┬──────────┬──────────┬──────────┬──────────┐
│ id │ name          │ namespace   │ version │ mode    │ pid      │ uptime │ ↺    │ status    │ cpu      │ mem      │ user     │ watching │
├────┼───────────────┼─────────────┼─────────┼─────────┼──────────┼────────┼──────┼───────────┼──────────┼──────────┼──────────┼──────────┤
│ 0  │ naive         │ default     │ N/A     │ fork    │ 1492097  │ 22h    │ 39   │ online    │ 0%       │ 8.1mb    │ root     │ disabled │
│ 1  │ naive-http    │ default     │ N/A     │ fork    │ 1492102  │ 22h    │ 9    │ online    │ 0%       │ 8.3mb    │ root     │ disabled │
└────┴───────────────┴─────────────┴─────────┴─────────┴──────────┴────────┴──────┴───────────┴──────────┴──────────┴──────────┴──────────┘

5.
每次执行pmr命令，检查用户目录下是否有.pmr 文件夹，如果没有，则创建，并在文件夹下创建dump.json文件，用于管理使用pmr命令启动的进程，例如:
[
    {
        "id": 1,
        "name": "test",
        "program": "python",
        "args": ["-m", "test"]
    },
    {
        "id": 2,
        "name": "test2",
        "program": "python",
        "args": ["-m", "test2"]
    }
]
修改list（ls）命令，添加参数 --system，用于显示系统所有进程，如果不添加该参数，只显示在~/.pmr/dump.json中的进程
增加stop命令，用于关闭指定进程，例如pmr stop 1，关闭~/.pmr/dump.json中id为1的进程，或者pmr stop test，关闭~/.pmr/dump.json中name为test的进程

6.
验证start命令成功执行notepad程序后，相关信息是否写入dump.json，并且执行stop命令能正常结束进程

7.继续修改pmr start命令，使得以下几种方式均可成功：
pmr start pmr_id // 启动存在于dump.json中的进程
pmr start name // 启动存在于dump.json中的进程
pmr start --config config.json // 从配置文件启动进程
pmr start program_name // 从程序启动进程

8.增加pmr stop命令，用于关闭指定进程，例如pmr stop 1，关闭~/.pmr/dump.json中id为1的进程，或者pmr stop test，关闭~/.pmr/dump.json中name为test的进程

9.增加命令：restart, 例如 pmr restart 1，重启~/.pmr/dump.json中id为1的进程，或者pmr restart test，重启~/.pmr/dump.json中name为test的进程, pmr restart --config config.json, 从配置文件重启进程