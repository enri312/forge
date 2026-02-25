package api;

import core.CoreService;
import database.DatabaseService;

public class Main {

    public static void main(String[] args) {
        System.out.println("API running!");
        System.out.println("Status: " + new DatabaseService().getStatus());
    }
}
