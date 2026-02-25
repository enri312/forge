package database;

import core.CoreService;

public class DatabaseService {

    public String getStatus() {
        return new CoreService().getName() + " is Connected";
    }
}
