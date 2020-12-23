set /p nginx_path=input your nginx path (such as D:\xxx\nginx): 
wmic ENVIRONMENT create name="NGINX_HOME",username="<system>",VariableValue="%nginx_path%"
wmic ENVIRONMENT where "name='path' and username='<system>'" set VariableValue='%path%;%nginx_path%'
pause